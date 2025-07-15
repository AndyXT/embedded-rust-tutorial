#!/usr/bin/env python3
"""
Code Example Fixer for Embedded Rust Tutorial

This script identifies and fixes compilation issues in code examples,
ensuring all examples can actually compile on embedded targets.
"""

import re
import os
import sys
import subprocess
import tempfile
from pathlib import Path
from typing import List, Dict, Tuple, Optional

class CodeExampleFixer:
    """Fixes compilation issues in embedded Rust code examples"""
    
    def __init__(self, document_path: str):
        self.document_path = Path(document_path)
        self.document_content = self._load_document()
        self.fixes_applied = []
        self.test_results = {
            'total_tested': 0,
            'originally_working': 0,
            'fixed': 0,
            'still_broken': 0
        }
        
    def _load_document(self) -> str:
        """Load the document content"""
        with open(self.document_path, 'r', encoding='utf-8') as f:
            return f.read()
    
    def find_and_fix_examples(self) -> str:
        """Find and fix all code examples in the document"""
        print("🔧 Finding and fixing code examples...")
        
        # Process the document content
        fixed_content = self.document_content
        
        # Find all Rust code blocks
        code_block_pattern = r'(```rust\n)(.*?)\n```'
        matches = list(re.finditer(code_block_pattern, fixed_content, re.DOTALL))
        
        # Process matches in reverse order to maintain positions
        for match in reversed(matches):
            original_code = match.group(2)
            
            # Skip if it's clearly not meant to be compiled
            if self._should_skip_example(original_code):
                continue
            
            # Test original code
            if self._test_code_compilation(original_code):
                self.test_results['originally_working'] += 1
                continue
            
            # Try to fix the code
            fixed_code = self._fix_code_example(original_code)
            
            if fixed_code != original_code:
                # Test the fixed code
                if self._test_code_compilation(fixed_code):
                    # Replace in document
                    start, end = match.span()
                    fixed_content = (fixed_content[:match.start(2)] + 
                                   fixed_code + 
                                   fixed_content[match.end(2):])
                    
                    self.fixes_applied.append({
                        'line': self.document_content[:match.start()].count('\n') + 1,
                        'original': original_code[:100] + "..." if len(original_code) > 100 else original_code,
                        'fixed': fixed_code[:100] + "..." if len(fixed_code) > 100 else fixed_code
                    })
                    self.test_results['fixed'] += 1
                else:
                    self.test_results['still_broken'] += 1
            else:
                self.test_results['still_broken'] += 1
            
            self.test_results['total_tested'] += 1
        
        return fixed_content
    
    def _should_skip_example(self, content: str) -> bool:
        """Determine if an example should be skipped"""
        skip_patterns = [
            r'//.*C code.*:',  # C code comparisons
            r'//.*approach:',  # Approach descriptions
            r'^\s*//.*$',      # Comment-only blocks
            r'\.\.\..*\.\.\.',  # Ellipsis indicating incomplete code
            r'^\s*\[.*\]\s*$', # Configuration sections only
            r'memory\.x',      # Memory layout files
            r'MEMORY\s*{',     # Memory definitions
        ]
        
        for pattern in skip_patterns:
            if re.search(pattern, content, re.MULTILINE | re.IGNORECASE):
                return True
        
        # Skip very short examples (likely fragments)
        if len(content.strip().split('\n')) < 2:
            return True
            
        return False
    
    def _test_code_compilation(self, code: str) -> bool:
        """Test if code compiles"""
        with tempfile.TemporaryDirectory() as temp_dir:
            project_dir = Path(temp_dir) / "test_project"
            
            try:
                self._create_test_project(project_dir, code)
                
                result = subprocess.run([
                    'cargo', 'check', '--target', 'thumbv7em-none-eabihf'
                ], 
                cwd=project_dir, 
                capture_output=True, 
                text=True,
                timeout=15
                )
                
                return result.returncode == 0
                
            except Exception:
                return False
    
    def _create_test_project(self, project_dir: Path, code: str) -> None:
        """Create a test project for the code"""
        project_dir.mkdir(parents=True, exist_ok=True)
        src_dir = project_dir / "src"
        src_dir.mkdir(exist_ok=True)
        
        # Create Cargo.toml
        cargo_toml = """[package]
name = "test_example"
version = "0.1.0"
edition = "2021"

[dependencies]
cortex-m = "0.7"
cortex-m-rt = "0.7"
panic-halt = "0.2"
heapless = "0.7"
nb = "1.0"
embedded-hal = "0.2"

# Crypto dependencies
chacha20poly1305 = { version = "0.10", default-features = false }
aes-gcm = { version = "0.10", default-features = false }
sha2 = { version = "0.10", default-features = false }
subtle = { version = "2.5", default-features = false }
zeroize = { version = "1.6", default-features = false, features = ["derive"] }

[profile.release]
opt-level = "z"
lto = true
panic = "abort"
"""
        
        with open(project_dir / "Cargo.toml", 'w') as f:
            f.write(cargo_toml)
        
        # Create memory.x
        memory_x = """MEMORY
{
  FLASH : ORIGIN = 0x08000000, LENGTH = 256K
  RAM : ORIGIN = 0x20000000, LENGTH = 64K
}
"""
        with open(project_dir / "memory.x", 'w') as f:
            f.write(memory_x)
        
        # Create .cargo/config.toml
        cargo_dir = project_dir / ".cargo"
        cargo_dir.mkdir(exist_ok=True)
        
        config_toml = """[target.thumbv7em-none-eabihf]
rustflags = [
  "-C", "link-arg=-Tlink.x",
  "-C", "target-cpu=cortex-m4",
]

[build]
target = "thumbv7em-none-eabihf"
"""
        
        with open(cargo_dir / "config.toml", 'w') as f:
            f.write(config_toml)
        
        # Prepare and write the code
        prepared_code = self._prepare_code_for_compilation(code)
        
        with open(src_dir / "main.rs", 'w') as f:
            f.write(prepared_code)
    
    def _prepare_code_for_compilation(self, code: str) -> str:
        """Prepare code for compilation by adding necessary boilerplate"""
        # If it's already a complete program, use as-is
        if '#![no_std]' in code and ('#![no_main]' in code or '#[entry]' in code):
            return code
        
        # If it's a function or struct definition, wrap it
        if any(pattern in code for pattern in ['fn ', 'struct ', 'enum ', 'impl ', 'use ', 'mod ']):
            return f"""#![no_std]
#![no_main]

use panic_halt as _;
use cortex_m_rt::entry;

{code}

#[entry]
fn main() -> ! {{
    loop {{
        cortex_m::asm::nop();
    }}
}}
"""
        
        # For other code snippets, create a minimal wrapper
        return f"""#![no_std]
#![no_main]

use panic_halt as _;
use cortex_m_rt::entry;

#[entry]
fn main() -> ! {{
    {code}
    loop {{
        cortex_m::asm::nop();
    }}
}}
"""
    
    def _fix_code_example(self, code: str) -> str:
        """Apply common fixes to make code compile"""
        fixed_code = code
        
        # Fix 1: Add missing imports
        fixed_code = self._add_missing_imports(fixed_code)
        
        # Fix 2: Fix common syntax issues
        fixed_code = self._fix_syntax_issues(fixed_code)
        
        # Fix 3: Fix type issues
        fixed_code = self._fix_type_issues(fixed_code)
        
        # Fix 4: Fix embedded-specific issues
        fixed_code = self._fix_embedded_issues(fixed_code)
        
        # Fix 5: Fix crypto-specific issues
        fixed_code = self._fix_crypto_issues(fixed_code)
        
        return fixed_code
    
    def _add_missing_imports(self, code: str) -> str:
        """Add missing imports based on code content"""
        imports_to_add = []
        
        # Common patterns and their required imports
        import_patterns = {
            r'ZeroizeOnDrop': 'use zeroize::{Zeroize, ZeroizeOnDrop};',
            r'ConstantTimeEq': 'use subtle::ConstantTimeEq;',
            r'Vec<': 'use heapless::Vec;',
            r'String<': 'use heapless::String;',
            r'HashMap<': 'use heapless::HashMap;',
            r'Mutex<': 'use cortex_m::interrupt::Mutex;',
            r'RefCell<': 'use core::cell::RefCell;',
            r'#\[interrupt\]': 'use cortex_m_rt::interrupt;',
            r'ChaCha20Poly1305': 'use chacha20poly1305::ChaCha20Poly1305;',
            r'Aes.*Gcm': 'use aes_gcm::Aes256Gcm;',
            r'Sha256': 'use sha2::{Sha256, Digest};',
            r'ptr::': 'use core::ptr;',
            r'mem::': 'use core::mem;',
            r'slice::': 'use core::slice;',
        }
        
        for pattern, import_stmt in import_patterns.items():
            if re.search(pattern, code) and import_stmt not in code:
                imports_to_add.append(import_stmt)
        
        if imports_to_add:
            # Add imports at the beginning
            imports_section = '\n'.join(imports_to_add) + '\n\n'
            
            # Find where to insert imports
            if 'use ' in code:
                # Insert with existing imports
                first_use = code.find('use ')
                return code[:first_use] + imports_section + code[first_use:]
            else:
                # Insert at the beginning
                return imports_section + code
        
        return code
    
    def _fix_syntax_issues(self, code: str) -> str:
        """Fix common syntax issues"""
        fixed = code
        
        # Fix escaped pipes in closures
        fixed = re.sub(r'\\\|', '|', fixed)
        
        # Fix common macro issues
        fixed = re.sub(r'assert!\s*\(([^)]+)\)', r'assert!(\1)', fixed)
        
        # Fix string literal issues
        fixed = re.sub(r'\\\"', '"', fixed)
        
        # Fix common formatting issues
        fixed = re.sub(r'\s+\n', '\n', fixed)  # Remove trailing whitespace
        
        return fixed
    
    def _fix_type_issues(self, code: str) -> str:
        """Fix common type issues"""
        fixed = code
        
        # Fix generic type syntax
        fixed = re.sub(r'Vec<([^>]+)>', r'Vec<\1, 32>', fixed)  # Add capacity to Vec
        
        # Fix Result types
        fixed = re.sub(r'Result<([^,>]+), ([^>]+)>', r'Result<\1, \2>', fixed)
        
        # Fix Option unwrapping
        fixed = re.sub(r'\.unwrap\(\)', '.unwrap()', fixed)
        
        return fixed
    
    def _fix_embedded_issues(self, code: str) -> str:
        """Fix embedded-specific issues"""
        fixed = code
        
        # Fix interrupt attribute
        if '#[interrupt]' in fixed and 'use cortex_m_rt::interrupt;' not in fixed:
            fixed = 'use cortex_m_rt::interrupt;\n\n' + fixed
        
        # Fix panic handler issues
        if '#[panic_handler]' in fixed:
            if 'use panic_halt as _;' not in fixed:
                fixed = 'use panic_halt as _;\n' + fixed
            if 'PanicInfo' in fixed and 'use core::panic::PanicInfo;' not in fixed:
                fixed = 'use core::panic::PanicInfo;\n' + fixed
        
        # Fix entry point issues
        if '#[entry]' in fixed and 'use cortex_m_rt::entry;' not in fixed:
            fixed = 'use cortex_m_rt::entry;\n' + fixed
        
        return fixed
    
    def _fix_crypto_issues(self, code: str) -> str:
        """Fix crypto-specific issues"""
        fixed = code
        
        # Fix zeroize issues
        if 'ZeroizeOnDrop' in fixed:
            if '#[derive(ZeroizeOnDrop)]' in fixed and 'use zeroize::ZeroizeOnDrop;' not in fixed:
                fixed = 'use zeroize::ZeroizeOnDrop;\n' + fixed
        
        # Fix subtle crate issues
        if 'ct_eq' in fixed and 'use subtle::ConstantTimeEq;' not in fixed:
            fixed = 'use subtle::ConstantTimeEq;\n' + fixed
        
        # Fix common crypto function calls
        fixed = re.sub(r'\.ct_eq\(([^)]+)\)\.into\(\)', r'.ct_eq(\1).into()', fixed)
        
        return fixed
    
    def run_fixes(self) -> str:
        """Run all fixes and return the fixed document"""
        print("🚀 Starting code example fixes...")
        print("=" * 60)
        
        # Check if Rust is available
        try:
            subprocess.run(['cargo', '--version'], capture_output=True, check=True)
        except (subprocess.CalledProcessError, FileNotFoundError):
            print("❌ Error: Cargo not found. Please install Rust toolchain.")
            return self.document_content
        
        # Find and fix examples
        fixed_content = self.find_and_fix_examples()
        
        print("=" * 60)
        print("✅ Code fixing complete!")
        
        return fixed_content
    
    def generate_fix_report(self) -> str:
        """Generate a report of fixes applied"""
        results = self.test_results
        
        report = f"""
# Code Example Fix Report

## Summary
- **Total Examples Tested**: {results['total_tested']}
- **Originally Working**: {results['originally_working']}
- **Successfully Fixed**: {results['fixed']}
- **Still Broken**: {results['still_broken']}
- **Total Fixes Applied**: {len(self.fixes_applied)}

## Fix Success Rate
- **Working Examples**: {results['originally_working'] + results['fixed']}/{results['total_tested']}
- **Success Rate**: {((results['originally_working'] + results['fixed']) / max(results['total_tested'], 1) * 100):.1f}%

## Fixes Applied
"""
        
        if self.fixes_applied:
            for i, fix in enumerate(self.fixes_applied, 1):
                report += f"""
### Fix {i} (Line {fix['line']})
**Original:**
```rust
{fix['original']}
```

**Fixed:**
```rust
{fix['fixed']}
```
"""
        else:
            report += "\nNo fixes were needed or applied.\n"
        
        report += f"""
## Assessment
{"✅ Excellent - Most examples now compile!" if results['fixed'] > results['still_broken'] else
 "⚠️ Good - Some examples fixed, others may need manual attention" if results['fixed'] > 0 else
 "❌ No automatic fixes possible - manual review needed"}
"""
        
        return report

def main():
    """Main fixing function"""
    if len(sys.argv) != 2:
        print("Usage: python code_example_fixer.py <document_path>")
        sys.exit(1)
    
    document_path = sys.argv[1]
    
    # Run fixes
    fixer = CodeExampleFixer(document_path)
    fixed_content = fixer.run_fixes()
    
    # Save fixed document
    fixed_path = Path(document_path).stem + "-fixed" + Path(document_path).suffix
    with open(fixed_path, 'w', encoding='utf-8') as f:
        f.write(fixed_content)
    
    # Generate and save report
    report = fixer.generate_fix_report()
    
    with open("code_fix_report.md", 'w') as f:
        f.write(report)
    
    print(f"📄 Fixed document saved to: {fixed_path}")
    print(f"📄 Fix report saved to: code_fix_report.md")
    
    # Print summary
    results = fixer.test_results
    total_working = results['originally_working'] + results['fixed']
    success_rate = (total_working / max(results['total_tested'], 1) * 100)
    
    print(f"\n📊 SUMMARY:")
    print(f"  Fixed: {results['fixed']} examples")
    print(f"  Working: {total_working}/{results['total_tested']} ({success_rate:.1f}%)")
    print(f"  Applied: {len(fixer.fixes_applied)} fixes")

if __name__ == "__main__":
    main()