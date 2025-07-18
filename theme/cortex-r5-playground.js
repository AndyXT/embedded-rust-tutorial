// Custom playground functionality for Cortex-R5 compilation
document.addEventListener('DOMContentLoaded', function() {
    // Find all Rust code blocks
    const codeBlocks = document.querySelectorAll('.language-rust');
    
    codeBlocks.forEach(block => {
        // Skip if already has buttons or is marked as ignore
        if (block.classList.contains('ignore') || block.querySelector('.buttons')) {
            return;
        }
        
        // Get the code content
        const code = block.querySelector('code').textContent;
        
        // Create a custom button container
        const buttonContainer = document.createElement('div');
        buttonContainer.className = 'buttons';
        buttonContainer.style.position = 'absolute';
        buttonContainer.style.right = '5px';
        buttonContainer.style.top = '5px';
        
        // Create "Copy for Cortex-R5" button
        const copyButton = document.createElement('button');
        copyButton.className = 'fa fa-copy clip-button';
        copyButton.title = 'Copy for Cortex-R5 project';
        copyButton.setAttribute('aria-label', 'Copy for Cortex-R5 project');
        
        copyButton.onclick = function() {
            // Prepare the code for Cortex-R5
            let cortexCode = code;
            
            // If it doesn't have #![no_std], add the boilerplate
            if (!code.includes('#![no_std]')) {
                cortexCode = `#![no_std]
#![no_main]

use panic_halt as _;
use core::result::Result;

${code}

#[no_mangle]
pub unsafe extern "C" fn Reset() -> ! {
    // Your code here
    loop {}
}`;
            }
            
            // Copy to clipboard
            navigator.clipboard.writeText(cortexCode).then(() => {
                copyButton.classList.add('clip-button-checked');
                setTimeout(() => {
                    copyButton.classList.remove('clip-button-checked');
                }, 1000);
            });
        };
        
        buttonContainer.appendChild(copyButton);
        
        // Add compilation instructions
        const infoDiv = document.createElement('div');
        infoDiv.className = 'cortex-r5-info';
        infoDiv.style.fontSize = '0.8em';
        infoDiv.style.color = '#666';
        infoDiv.style.marginTop = '5px';
        infoDiv.innerHTML = `
            <details>
                <summary style="cursor: pointer;">📋 How to compile for Cortex-R5</summary>
                <pre style="font-size: 0.9em; background: #f5f5f5; padding: 10px; margin-top: 5px;">
# Create a new project
cargo new --bin my_cortex_r5_app
cd my_cortex_r5_app

# Add target
rustup target add armv7r-none-eabi

# Update Cargo.toml with dependencies:
# [dependencies]
# cortex-r-rt = "0.1"
# panic-halt = "0.2"
# heapless = "0.8"

# Create memory.x file (see Environment Setup section)

# Build
cargo build --target armv7r-none-eabi --release
                </pre>
            </details>
        `;
        
        // Position the code block container
        const container = block.parentElement;
        container.style.position = 'relative';
        container.appendChild(buttonContainer);
        container.appendChild(infoDiv);
    });
});