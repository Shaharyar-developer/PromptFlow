A command-line tool for generating detailed anime-style AI image prompts using Google's Gemini AI model.

## Overview

PromptFlow takes simple keyword inputs and transforms them into comprehensive, detailed prompts specifically optimized for anime-style AI image generation. It uses the Gemini API to create prompts with proper weighting, segmentation, and artistic direction.

## Features

- Generates detailed anime-style prompts from simple keywords
- Includes advanced prompt techniques like keyword weighting `(keyword:factor)` and segmentation with `BREAK`
- Maintains a history of previous prompts for context
- Automatically provides a standard negative prompt
- Securely manages API keys

## Installation

### Prerequisites

- Rust and Cargo installed

### Building from source

```bash
git clone https://github.com/Shaharyar-developer/PromptFlow.git
cd PromptFlow
makepkg -si
```

The executable will be available at PromptFlow.

## Usage

```bash
PromptFlow "your keyword"
```

### Command-line Options

- `--key` or `-k`: Provide your Gemini API key
- `--prompt` or `-p`: Specify the input keyword
- Direct input: Simply provide your keyword as the first argument

### API Key Management

The application will look for your API key in the following order:

1. Temporary file in your system's temp directory
2. Command-line argument (`--key` or `-k`)
3. Environment variable (`GENAI_API_KEY`)

Once provided, the key will be stored in a temporary file for future use.

## Examples

```bash
# Basic usage
PromptFlow "anime knight defending a gate"

# Providing API key
PromptFlow --key YOUR_API_KEY "magical girl transformation"

# Using named prompt parameter
PromptFlow -p "cyberpunk samurai"
```

## Output

The tool generates and displays:

1. A detailed prompt optimized for anime-style image generation
2. A standard negative prompt to avoid common AI image generation issues

## Dependencies

- gemini_rs - For interacting with Google's Gemini API
- credentials - For secure credential management
- tokio - For asynchronous runtime

## License

See the LICENSE file for details.
