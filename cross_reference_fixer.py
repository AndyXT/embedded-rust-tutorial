#!/usr/bin/env python3
"""
Cross-Reference Fixer for mdBook Conversion

This script fixes broken cross-references by updating anchor-only links
to point to the correct files in the mdBook structure.
"""

import re
from pathlib import Path
from typing import Dict, List, Tuple


class CrossReferenceFixer:
    """Fixes cross-references in mdBook structure."""
    
    def __init__(self, src_dir: str = "src"):
        self.src_dir = Path(src_dir)
        
        # Mapping of anchor names to their actual file locations
        self.anchor_to_file_map = {
            # Cryptography section anchors
            "constant-time-implementations": "cryptography/constant-time.md",
            "secure-coding-patterns": "cryptography/secure-patterns.md",
            "key-management-and-zeroization": "cryptography/key-management.md",
            "hardware-crypto-acceleration": "cryptography/hardware-crypto.md",
            "side-channel-mitigations": "cryptography/side-channels.md",
            
            # Core concepts anchors
            "functional-programming-and-data-processing": "core-concepts/functional.md",
            "advanced-type-system-features": "core-concepts/advanced-types.md",
            "memory-model-differences": "core-concepts/memory-model.md",
            "ownership-and-memory-management": "core-concepts/ownership.md",
            "error-handling-without-exceptions": "core-concepts/error-handling.md",
            "type-system-overview": "core-concepts/type-system.md",
            "type-system-advantages-for-security": "core-concepts/type-system-advantages-for-security.md",
            "safety-guarantees-for-crypto": "core-concepts/safety.md",
            
            # Migration section anchors
            "incremental-migration-strategies": "migration/strategies.md",
            "ffi-integration-with-c-libraries": "migration/ffi-integration.md",
            "testing-and-validation": "migration/testing.md",
            "debugging-and-tooling": "migration/debugging.md",
            "performance-considerations": "migration/performance.md",
            
            # Embedded patterns anchors
            "no-std-programming-essentials": "embedded-patterns/no-std.md",
            "hardware-abstraction-patterns": "embedded-patterns/hardware-abstraction.md",
            "interrupt-handling": "embedded-patterns/interrupts.md",
            "static-memory-management": "embedded-patterns/static-memory.md",
            "dma-and-hardware-integration": "embedded-patterns/dma-integration.md",
            
            # Quick reference anchors
            "c-to-rust-syntax-mapping": "quick-reference/syntax-mapping.md",
            "memory-and-pointer-patterns": "quick-reference/memory-patterns.md",
            "control-flow-and-functions": "quick-reference/control-flow.md",
            "error-handling-patterns": "quick-reference/error-handling.md",
            "crypto-specific-quick-reference": "quick-reference/crypto-reference.md",
            "embedded-specific-quick-reference": "quick-reference/embedded-reference.md",
            "critical-differences-and-gotchas": "quick-reference/gotchas.md",
            
            # Environment setup anchors
            "rust-installation-and-toolchain": "environment-setup/installation.md",
            "target-configuration": "environment-setup/target-config.md",
            "project-structure-and-dependencies": "environment-setup/project-structure.md",
            "build-configuration": "environment-setup/build-config.md",
            "verification-and-testing": "environment-setup/verification.md",
        }
    
    def fix_all_cross_references(self) -> None:
        """Fix all cross-references in the mdBook structure."""
        print("Fixing cross-references...")
        
        fixes_applied = 0
        files_updated = 0
        
        for md_file in self.src_dir.rglob("*.md"):
            if md_file.is_file():
                file_fixes = self._fix_file_cross_references(md_file)
                if file_fixes > 0:
                    files_updated += 1
                    fixes_applied += file_fixes
        
        print(f"Fixed {fixes_applied} cross-references in {files_updated} files")
    
    def _fix_file_cross_references(self, file_path: Path) -> int:
        """Fix cross-references in a single file."""
        try:
            with open(file_path, 'r', encoding='utf-8') as f:
                content = f.read()
            
            original_content = content
            fixes_count = 0
            
            # Find all anchor-only links
            anchor_pattern = re.compile(r'\[([^\]]+)\]\(#([a-zA-Z0-9_-]+)\)')
            
            for match in anchor_pattern.finditer(original_content):
                link_text = match.group(1)
                anchor = match.group(2)
                
                # Check if this anchor should point to another file
                if anchor in self.anchor_to_file_map:
                    target_file = self.anchor_to_file_map[anchor]
                    relative_path = self._get_relative_path(file_path, target_file)
                    
                    old_link = f"[{link_text}](#{anchor})"
                    new_link = f"[{link_text}]({relative_path})"
                    
                    content = content.replace(old_link, new_link)
                    fixes_count += 1
                    
                    print(f"  {file_path.relative_to(self.src_dir)}: {old_link} -> {new_link}")
            
            # Write updated content if changes were made
            if content != original_content:
                with open(file_path, 'w', encoding='utf-8') as f:
                    f.write(content)
            
            return fixes_count
            
        except Exception as e:
            print(f"Error fixing cross-references in {file_path}: {e}")
            return 0
    
    def _get_relative_path(self, source_file: Path, target_file: str) -> str:
        """Get relative path from source file to target file."""
        source_relative = source_file.relative_to(self.src_dir)
        source_dir = source_relative.parent
        
        # Calculate relative path
        if str(source_dir) == '.':
            # Source is in root, target path as-is
            return target_file
        else:
            # Source is in subdirectory, need to go up
            levels_up = len(source_dir.parts)
            prefix = '../' * levels_up
            return f"{prefix}{target_file}"
    
    def add_missing_anchors(self) -> None:
        """Add missing anchors to files that should have them."""
        print("Adding missing anchors...")
        
        # Files that need specific anchors added
        anchor_additions = {
            "core-concepts/functional.md": [
                ("# 3.5 Functional Programming and Data Processing", "functional-programming-and-data-processing")
            ],
            "core-concepts/advanced-types.md": [
                ("# 3.4 Advanced Type System Features", "advanced-type-system-features")
            ],
            "core-concepts/memory-model.md": [
                ("# Memory Model Differences", "memory-model-differences")
            ],
            "cryptography/constant-time.md": [
                ("# 5.2 Constant-Time Implementations", "constant-time-implementations")
            ],
            "cryptography/secure-patterns.md": [
                ("# 5.1 Secure Coding Patterns", "secure-coding-patterns")
            ],
            "cryptography/key-management.md": [
                ("# 5.3 Key Management and Zeroization", "key-management-and-zeroization")
            ],
            "migration/strategies.md": [
                ("# 6.1 Incremental Migration Strategies", "incremental-migration-strategies")
            ],
            "migration/ffi-integration.md": [
                ("# 6.2 FFI Integration with C Libraries", "ffi-integration-with-c-libraries")
            ]
        }
        
        for file_path, anchors in anchor_additions.items():
            self._add_anchors_to_file(file_path, anchors)
    
    def _add_anchors_to_file(self, file_path: str, anchors: List[Tuple[str, str]]) -> None:
        """Add anchors to a specific file."""
        full_path = self.src_dir / file_path
        
        if not full_path.exists():
            print(f"File not found: {file_path}")
            return
        
        try:
            with open(full_path, 'r', encoding='utf-8') as f:
                content = f.read()
            
            original_content = content
            
            for header_text, anchor_id in anchors:
                # Check if header exists and doesn't already have an anchor
                if header_text in content and f"{{{anchor_id}}}" not in content:
                    # Add anchor to the header
                    new_header = f"{header_text} {{{anchor_id}}}"
                    content = content.replace(header_text, new_header)
                    print(f"  Added anchor to {file_path}: {anchor_id}")
            
            if content != original_content:
                with open(full_path, 'w', encoding='utf-8') as f:
                    f.write(content)
                    
        except Exception as e:
            print(f"Error adding anchors to {file_path}: {e}")
    
    def validate_cross_references(self) -> None:
        """Validate that all cross-references are working."""
        print("Validating cross-references...")
        
        # Re-run the link validator to check results
        from link_validator import LinkValidator
        
        validator = LinkValidator()
        validator.scan_all_files()
        validator.validate_all_links()
        
        broken_count = len(validator.broken_links)
        total_count = len(validator.all_links)
        
        print(f"Validation results: {total_count - broken_count}/{total_count} links valid")
        
        if broken_count > 0:
            print(f"Remaining broken links: {broken_count}")
            for link in validator.broken_links[:5]:  # Show first 5
                print(f"  {link.source_file}:{link.line_number} - {link.error_message}")


def main():
    """Main function to fix cross-references."""
    fixer = CrossReferenceFixer()
    
    # Fix cross-references
    fixer.fix_all_cross_references()
    
    # Add missing anchors
    fixer.add_missing_anchors()
    
    # Validate results
    fixer.validate_cross_references()


if __name__ == "__main__":
    main()