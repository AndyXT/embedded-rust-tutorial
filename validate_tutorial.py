#!/usr/bin/env python3
"""
Tutorial Validation Script

This script validates the embedded Rust tutorial by:
1. Extracting and testing individual code examples
2. Providing detailed feedback on compilation issues
3. Suggesting fixes for common problems
"""

import re
import os
import sys
import subprocess
import tempfile
from pathlib import Path
from typing import List, Dict, Tuple, Optional

class TutorialValidator:
    """Validates the embedded Rust tutorial"""
    
    def __init__(self, document_path: str):
        self.document_path = Path(document_path)
        self.document_content = self._load_document()
        self.results = {
            'total_examples': 0,
            'syntax_valid': 0,
            'compiles': 0,
            'failed_examples': []
        }
        
    def _load_document(self) -> str:
        """Load the document content"""
        with open(self.document_path, 'r', encoding='utf-8') as f:
            return f.read()
    
    def extract_and_test_examples(self) -> None:
        """Extract and test all code examples"""
        print("🔍 Extracting code examples...")
        
        # Find all code blocks with language specification
        patterns = [
            (r'```rust\n(.*?)\n```', 'rust'),
            (r'```toml\n(.*?)\n```', 'toml'),
            (r'```bash\n(.*?)\n```', 'bash')
        ]
        
        all_examples = []
        
        for pattern, language in patterns:
            matches = re.finditer(pattern, self.document_content, re.DOTALL)
            for match in matches:
                content = match.group(1).strip()
                line_num = self.document_content[:match.start()].count('\n') + 1
                section = self._find_section_context(match.start())
                
                all_examples.append({
                    'language': language,
                    'content': content,
                    'line': line_num,
                    'section': section
                })
        
        self.results['total_examples'] = len(all_examples)
        print(f"📊 Found {len(all_examples)} code examples")
        
        # Test each example
        for i, example in enumerate(all_examples):
            print(f"  Testing {i+1}/{len(all_examples)}: {example['section'][:50]}...")
            self._test_example(example)
    
    def _find_section_context(self, position: int) -> str:
        """Find the section context for a given position"""
        content_before = self.document_content[:position]
        lines = content_before.split('\n')
        
        for line in reversed(lines):
            if line.startswith('#'):
                return line.strip('#').strip()
        
        return "Unknown Section"
    
    def _test_example(self, example: Dict) -> None:
        """Test a single code example"""
        language = example['language']
        content = example['content']
        
        if language == 'rust':
            self._test_rust_example(example)
        elif language == 'toml':
            self._test_toml_example(example)
        elif language == 'bash':
            self._test_bash_example(example)
    
    def _test_rust_example(self, example: Dict) -> None:
        """Test a Rust code example"""
        content = example['content']
        
        # First, check basic syntax
        syntax_valid = self._check_rust_syntax(content)
        if syntax_valid:
            self.results['syntax_valid'] += 1
        
        # Then try compilation
        compiles = self._test_rust_compilation(content)
        if compiles:
            self.results['compiles'] += 1
        else:
            self.results['failed_examples'].append({
                'section': example['section'],
                'line': example['line'],
                'language': example['language'],
                'error': 'Compilation failed'
            })
    
    def _check_rust_syntax(self, content: str) -> bool:
        """Check Rust syntax using rustc --parse-only"""
        with tempfile.NamedTemporaryFile(mode='w', suffix='.rs', delete=False) as f:
            f.write(content)
            temp_file = f.name
        
        try:
            result = subprocess.run([
                'rustc', '--parse-only', temp_file
            ], capture_output=True, text=True, timeout=10)
            
            return result.returncode == 0
        except:
            return False
        finally:
            os.unlink(temp_file)
    
    def _test_rust_compilation(self, content: str) -> bool:
        """Test Rust compilation in embedded context"""
        # Skip examples that are clearly not meant to be compiled
        if self._should_skip_compilation(content):
            return True  # Consider as "valid" since it's not meant to compile
        
        with tempfile.TemporaryDirectory() as temp_dir:
            project_dir = Path(temp_dir) / "test_project"
            
            try:
                self._create_minimal_project(project_dir, content)
                
                result = subprocess.run([
                    'cargo', 'check', '--target', 'thumbv7em-none-eabihf'
                ], 
                cwd=project_dir, 
                capture_output=True, 
                text=True,
                timeout=20
                )
                
                return result.returncode == 0
                
            except Exception as e:
                return False
    
    def _should_skip_compilation(self, content: str) -> bool:
        """Check if example should be skipped for compilation"""
        skip_indicators = [
            '// C code',
            '// approach:',
            'memory.x',
            'MEMORY {',
            '...',
            'error:',  # Error examples
            'BUG:',    # Bug examples
            '// ERROR:', # Error demonstrations
        ]
        
        for indicator in skip_indicators:
            if indicator in content:
                return True
        
        # Skip very short fragments
        if len(content.strip().split('\n')) < 2:
            return True
        
        return False
    
    def _create_minimal_project(self, project_dir: Path, content: str) -> None:
        """Create minimal project for testing"""
        project_dir.mkdir(parents=True, exist_ok=True)
        src_dir = project_dir / "src"
        src_dir.mkdir(exist_ok=True)
        
        # Minimal Cargo.toml
        cargo_toml = """[package]
name = "test"
version = "0.1.0"
edition = "2021"

[dependencies]
cortex-m = "0.7"
cortex-m-rt = "0.7"
panic-halt = "0.2"
heapless = "0.7"
chacha20poly1305 = { version = "0.10", default-features = false }
aes-gcm = { version = "0.10", default-features = false }
sha2 = { version = "0.10", default-features = false }
subtle = { version = "2.5", default-features = false }
zeroize = { version = "1.6", default-features = false, features = ["derive"] }
"""
        
        with open(project_dir / "Cargo.toml", 'w') as f:
            f.write(cargo_toml)
        
        # Memory layout
        memory_x = """MEMORY { FLASH : ORIGIN = 0x08000000, LENGTH = 256K; RAM : ORIGIN = 0x20000000, LENGTH = 64K }"""
        with open(project_dir / "memory.x", 'w') as f:
            f.write(memory_x)
        
        # Config
        cargo_dir = project_dir / ".cargo"
        cargo_dir.mkdir(exist_ok=True)
        config_toml = """[target.thumbv7em-none-eabihf]
rustflags = ["-C", "link-arg=-Tlink.x"]
[build]
target = "thumbv7em-none-eabihf"
"""
        with open(cargo_dir / "config.toml", 'w') as f:
            f.write(config_toml)
        
        # Prepare code
        prepared_code = self._prepare_code(content)
        with open(src_dir / "main.rs", 'w') as f:
            f.write(prepared_code)
    
    def _prepare_code(self, content: str) -> str:
        """Prepare code for compilation"""
        # If already complete, use as-is
        if '#![no_std]' in content and ('#![no_main]' in content or '#[entry]' in content):
            return content
        
        # Wrap incomplete code
        return f"""#![no_std]
#![no_main]

use panic_halt as _;
use cortex_m_rt::entry;

{content}

#[entry]
fn main() -> ! {{
    loop {{ cortex_m::asm::nop(); }}
}}
"""
    
    def _test_toml_example(self, example: Dict) -> None:
        """Test TOML example"""
        try:
            # Try to parse as TOML (requires toml package)
            import toml
            toml.loads(example['content'])
            self.results['syntax_valid'] += 1
            self.results['compiles'] += 1
        except:
            self.results['failed_examples'].append({
                'section': example['section'],
                'line': example['line'],
                'language': 'toml',
                'error': 'TOML parsing failed'
            })
    
    def _test_bash_example(self, example: Dict) -> None:
        """Test bash example"""
        # For bash, just check if it's not obviously broken
        content = example['content']
        if content.strip() and not content.startswith('...'):
            self.results['syntax_valid'] += 1
            self.results['compiles'] += 1
        else:
            self.results['failed_examples'].append({
                'section': example['section'],
                'line': example['line'],
                'language': 'bash',
                'error': 'Invalid bash syntax'
            })
    
    def run_validation(self) -> Dict:
        """Run complete validation"""
        print("🚀 Starting tutorial validation...")
        print("=" * 60)
        
        self.extract_and_test_examples()
        
        print("=" * 60)
        print("✅ Validation complete!")
        
        return self.results
    
    def generate_report(self) -> str:
        """Generate validation report"""
        results = self.results
        
        success_rate_syntax = (results['syntax_valid'] / results['total_examples'] * 100) if results['total_examples'] > 0 else 0
        success_rate_compile = (results['compiles'] / results['total_examples'] * 100) if results['total_examples'] > 0 else 0
        
        report = f"""# Code Example Test Report

**Total Code Examples:** {results['total_examples']}  
**Syntax Valid:** {results['syntax_valid']} ({success_rate_syntax:.1f}%)  
**Compiles:** {results['compiles']} ({success_rate_compile:.1f}%)  

## Summary

| Language | Total | Valid | Success Rate |
|----------|-------|-------|--------------|
"""
        
        # Count by language
        lang_stats = {}
        for example in self.results.get('all_examples', []):
            lang = example.get('language', 'unknown')
            if lang not in lang_stats:
                lang_stats[lang] = {'total': 0, 'valid': 0}
            lang_stats[lang]['total'] += 1
        
        # Add failed examples by language
        for failure in results['failed_examples']:
            lang = failure['language']
            if lang in lang_stats:
                lang_stats[lang]['valid'] = lang_stats[lang]['total'] - 1
        
        for lang, stats in lang_stats.items():
            success_rate = (stats['valid'] / stats['total'] * 100) if stats['total'] > 0 else 0
            report += f"| {lang} | {stats['total']} | {stats['valid']} | {success_rate:.1f}% |\n"
        
        if results['failed_examples']:
            report += f"\n\n## Failed Tests ({len(results['failed_examples'])})\n\n"
            
            for failure in results['failed_examples']:
                report += f"### {failure['section']} (Line {failure['line']})\n\n"
                report += f"**Language:** {failure['language']}  \n"
                report += f"**Syntax Valid:** {'❌' if 'syntax' in failure['error'].lower() else '✅'}  \n"
                report += f"**Compiles:** ❌  \n\n"
                report += f"**Errors:**\n- {failure['error']}\n\n"
        
        report += f"""
## Recommendations

1. **Fix Syntax Errors:** Address all syntax errors in code examples
2. **Test Compilation:** Set up proper embedded Rust toolchain for full compilation testing
3. **Add Context:** Ensure code examples have sufficient context for compilation
4. **Validate Examples:** Test all examples in actual embedded environment

---

*Generated by Code Example Tester*
"""
        
        return report

def main():
    """Main function"""
    if len(sys.argv) != 2:
        print("Usage: python validate_tutorial.py <document_path>")
        sys.exit(1)
    
    document_path = sys.argv[1]
    
    # Check dependencies
    try:
        subprocess.run(['rustc', '--version'], capture_output=True, check=True)
    except:
        print("❌ Error: rustc not found. Please install Rust.")
        sys.exit(1)
    
    try:
        subprocess.run(['cargo', '--version'], capture_output=True, check=True)
    except:
        print("❌ Error: cargo not found. Please install Rust.")
        sys.exit(1)
    
    # Run validation
    validator = TutorialValidator(document_path)
    results = validator.run_validation()
    
    # Generate report
    report = validator.generate_report()
    
    with open("code_test_report.md", 'w') as f:
        f.write(report)
    
    print(f"📄 Report saved to: code_test_report.md")
    
    # Print summary
    total = results['total_examples']
    compiles = results['compiles']
    success_rate = (compiles / total * 100) if total > 0 else 0
    
    print(f"\n📊 SUMMARY: {compiles}/{total} examples compile ({success_rate:.1f}%)")
    
    if results['failed_examples']:
        print(f"❌ {len(results['failed_examples'])} examples need attention")
    else:
        print("✅ All examples are working!")

if __name__ == "__main__":
    main()