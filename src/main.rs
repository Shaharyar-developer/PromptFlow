use credentials;
use gemini_rs;
use std::env;
use std::path::PathBuf;

/// System instructions for the Gemini AI model that define how to generate anime-style prompts
/// This multi-paragraph text guides the AI to create detailed anime-specific prompts with:
/// - Required components (subject, medium, style, etc.)
/// - Keyword weighting techniques
/// - Character consistency guidelines
/// - Prompt segmentation using BREAK
/// - Examples of properly formatted prompts
const SYSTEM_INSTRUCTION: &str = r#"
You are an assistant specialized in generating prompts **exclusively for anime-style** AI image generation from a given keyword.

**Core Task:**
Generate detailed AI image prompts based on a user's keyword, ensuring the final image aesthetic is distinctly **anime or manga style**.
**Crucially, you MUST actively utilize ALL the following techniques where appropriate to achieve high-quality anime results:**
*   Incorporate detailed keywords covering the 8 mandatory component categories, tailoring them for anime.
*   Employ keyword weighting `(keyword: factor)` to emphasize or de-emphasize specific anime elements (e.g., `(cel shading:1.3)`, `(sparkles:0.8)`).
*   Use known anime/manga character names for consistency when relevant to the keyword (e.g., 'Asuka Langley Soryu', 'Naruto Uzumaki').
*   Utilize the `BREAK` keyword for segmentation to prevent concept mixing in complex anime scenes.
*   Adhere to the principle of being highly detailed and specific to effectively guide the image generation process towards the desired anime look.

**Constraint:**
**Your primary focus is the anime aesthetic. Do NOT generate prompts aiming for realism, photorealism, or photographic styles. Avoid keywords like 'photo', 'photorealistic', 'hyperrealistic', 'realistic' unless used carefully as a minor modifier for specific background elements *while maintaining an overall anime style*.**

**Mandatory Prompt Components (Anime Focused):**
The prompts you generate MUST contain keywords covering the following categories, interpreted through an anime lens:
1.  **Subject:** (e.g., anime girl, shonen protagonist, mecha, fantasy creature in anime style)
2.  **Medium:** (e.g., anime screenshot, digital painting (anime style), manga page, light novel illustration, 2D animation cel, cel shading)
3.  **Style:** (e.g., modern anime, 90s anime aesthetic, shojo manga style, studio ghibli inspired, Makoto Shinkai style, chibi)
4.  **Art-sharing website/Platform:** (e.g., Pixiv, ArtStation (with anime tags), Danbooru aesthetic - *use platforms known for anime art*)
5.  **Resolution/Quality:** (e.g., high quality illustration, sharp focus, detailed linework, 4k anime wallpaper)
6.  **Additional details:** (background, clothing specific to anime tropes, actions, specific visual elements like speed lines, sparkles, dramatic expressions)
7.  **Color:** (e.g., vibrant anime colors, pastel palette, specific character hair/eye colors, cel shaded colors)
8.  **Lighting:** (e.g., dramatic anime lighting, volumetric light, rim lighting, soft anime glow, lens flare)

--------------------
**Example (Illustrating Anime Techniques):**

*   **Input Keyword:** 'Anime knight defending a gate'
*   **Generated Prompt:** '(epic male anime knight:1.2) with silver armor and (glowing blue sword:1.1), determined expression, dynamic action pose defending ancient stone gate BREAK dramatic background with stormy clouds and distant mountains, modern anime style, (cel shading:1.3), digital painting, featured on Pixiv, high quality illustration, sharp focus on knight, detailed armor design, cool color palette (blues, grays, silver:1.1), dramatic cinematic lighting, (rain effects:0.9), intense atmosphere, (fantasy anime aesthetic:1.2)'
    *   *Note:* This example uses anime-specific terms (anime knight, cel shading, Pixiv, fantasy anime aesthetic), weighting, the `BREAK` keyword, and covers all 8 component categories within the anime context.

--------------------
**Advanced Techniques Explained:**

**1. Keyword Weighting:**
*   Adjust the importance of a keyword using the syntax: `(keyword: factor)`
*   `factor < 1`: Less important (e.g., `(background details: 0.7)`)
*   `factor > 1`: More important (e.g., `(dynamic pose: 1.4)`)
*   *Use this to fine-tune specific anime elements.*

**2. Character Consistency:**
*   For consistent depictions, use known anime/manga character names when appropriate.
*   Example: Prompting for 'Rem' (from Re:Zero) helps generate her specific appearance.

**3. Prompt Segmentation (`BREAK`):**
*   Prevent the AI from mixing distinct concepts (e.g., applying character's hair color to the background). Separate using `BREAK` on its own line.
*   Example:
    anime girl with pink hair, wearing school uniform
    BREAK
    detailed classroom background, sunny day

--------------------
**Underlying Principle (Think like Stable Diffusion for Anime):**

*   Stable Diffusion is an image sampler. Your prompt guides it towards the *anime* part of its potential outputs.
*   **Detailed and specific prompts using techniques like weighting and segmentation are effective** because they narrow the sampling space, guiding diffusion towards the desired, complex **anime aesthetic**. Your role is to use *all* these tools to create the best guidance for generating anime-style images.
"#;

/// Standard negative prompt used for AI image generation
/// Contains terms to avoid common AI image generation issues like poor anatomy,
/// watermarks, low quality, etc.
const NEGATIVE_PROMPT: &str = "ugly, tiling, poorly drawn hands, poorly drawn feet, poorly drawn face, out of frame, extra limbs, disfigured, deformed, body out of frame, bad anatomy, watermark, signature, cut off, low contrast, underexposed, overexposed, bad art, beginner, amateur, distorted face, blurry, lowres, low quality, worst quality, low quality, normal quality, jpeg artifacts, signature, watermark, username, blurry";

// Model name for the Gemini API
const MODEL: &str = "gemini-2.0-flash";
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    let mut i = 1;
    let mut api_key = None;
    let mut prompt = None;

    // Log if no arguments were provided
    if args.len() < 2 {
        eprintln!("Error: No arguments provided. Please specify a prompt.");
        eprintln!(
            "Usage: {} [--key|-k KEY] [--prompt|-p PROMPT] or {} \"your prompt\"",
            args[0], args[0]
        );
        return Err("Missing arguments".into());
    }

    while i < args.len() {
        match args[i].as_str() {
            "--key" | "-k" => {
                if i + 1 < args.len() {
                    api_key = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    eprintln!("Error: API key argument requires a value");
                    eprintln!("Usage: {} --key YOUR_API_KEY", args[0]);
                    return Err("Missing API key value".into());
                }
            }
            "--prompt" | "-p" => {
                if i + 1 < args.len() {
                    prompt = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    eprintln!("Error: Prompt argument requires a value");
                    eprintln!("Usage: {} --prompt \"your prompt\"", args[0]);
                    return Err("Missing prompt value".into());
                }
            }
            _ => {
                // First non-flag argument is treated as the prompt
                if prompt.is_none() {
                    prompt = Some(args[i].clone());
                }
                i += 1;
            }
        }
    }

    // === API KEY MANAGEMENT ===
    // First check if key exists in temp file before requiring it as an argument
    let temp_path: PathBuf = env::temp_dir().join("key");
    let key = if temp_path.exists() {
        match std::fs::read_to_string(&temp_path) {
            Ok(contents) if !contents.trim().is_empty() => contents.trim().to_string(),
            _ => {
                // File exists but is empty or unreadable, try argument or env var
                match api_key {
                    Some(k) => k,
                    None => match credentials::var("GENAI_API_KEY") {
                        Ok(k) => {
                            // Write key to temp file for future use
                            std::fs::write(&temp_path, &k)?;
                            k
                        }
                        Err(_) => {
                            eprintln!(
                                "Error: API key not found. Provide it with --key or set GENAI_API_KEY environment variable"
                            );
                            return Err("Missing API key".into());
                        }
                    },
                }
            }
        }
    } else {
        // File doesn't exist, try argument or env var
        match api_key {
            Some(k) => {
                // Write key to temp file for future use
                std::fs::write(&temp_path, &k)?;
                k
            }
            None => match credentials::var("GENAI_API_KEY") {
                Ok(k) => {
                    // Write key to temp file for future use
                    std::fs::write(&temp_path, &k)?;
                    k
                }
                Err(_) => {
                    eprintln!(
                        "Error: API key not found. Provide it with --key or set GENAI_API_KEY environment variable"
                    );
                    return Err("Missing API key".into());
                }
            },
        }
    };

    // Initialize Gemini API client with the API key
    let client = gemini_rs::Client::new(key);

    // === USER INPUT HANDLING ===
    // Get the prompt from earlier parsing
    let prompt = match prompt {
        Some(p) if !p.trim().is_empty() => p.trim().to_string(),
        _ => {
            eprintln!("Error: Please provide a non-empty prompt");
            eprintln!("Usage: {} \"your prompt\"", args[0]);
            return Err("Empty prompt".into());
        }
    };

    // === PROMPT HISTORY MANAGEMENT ===
    // Load or create prompt history file in temp directory
    let history_path: PathBuf = env::temp_dir().join("prompt_history");
    let mut history = if history_path.exists() {
        match std::fs::read_to_string(&history_path) {
            Ok(contents) => contents,
            Err(_) => String::new(),
        }
    } else {
        String::new()
    };

    // Append current prompt to history
    history.push_str(&format!("{}\n", prompt));
    std::fs::write(&history_path, &history)?;

    // Extract the 5 most recent prompts to provide context to the AI
    let recent_prompts: String = history
        .lines()
        .collect::<Vec<&str>>()
        .into_iter()
        .rev() // Reverse to get the most recent first
        .take(5) // Take only the 5 most recent entries
        .rev() // Reverse back to chronological order
        .collect::<Vec<&str>>()
        .join("\n");

    // Combine system instructions with recent prompt history
    let system_instruction = format!(
        "{}\n\n--------------------\n**Previous Generated Prompts:**\n{}",
        SYSTEM_INSTRUCTION, recent_prompts
    );

    // === AI PROMPT GENERATION ===
    println!("Generating prompt for: {:?}", prompt);

    // Call the Gemini API to generate a detailed anime prompt
    let res = client
        .chat(MODEL) // Using the flash model for faster response
        .system_instruction(&system_instruction) // Pass system instructions and history
        .send_message(&prompt) // Send the user's keyword
        .await?;
    let text = res.to_string();

    // === OUTPUT RESULTS ===
    // Display the generated prompt and standard negative prompt with clear formatting
    println!("\n=== GENERATED PROMPT ===");
    println!("{}", text);
    println!("\n=== NEGATIVE PROMPT ===");
    println!("{}", NEGATIVE_PROMPT);
    Ok(())
}
