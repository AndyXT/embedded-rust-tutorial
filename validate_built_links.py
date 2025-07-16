#!/usr/bin/env python3
"""
Validate links in the built mdBook HTML output.

This script checks that all internal links in the built HTML work correctly.
"""

import os
import re
from pathlib import Path
from urllib.parse import unquote
from typing import Set, List, Tuple


class BuiltLinkValidator:
    """Validates links in built mdBook HTML."""
    
    def __init__(self, book_dir: str = "book-test"):
        self.book_dir = Path(book_dir)
        self.html_files: Set[str] = set()
        self.broken_links: List[Tuple[str, str, str]] = []
        
    def validate_all_links(self) -> bool:
        """Validate all links in the built book."""
        print("Validating links in built mdBook...")
        
        # Find all HTML files
        self._find_html_files()
        print(f"Found {len(self.html_files)} HTML files")
        
        # Validate links in each file
        for html_file in self.html_files:
            self._validate_file_links(html_file)
        
        # Report results
        total_files = len(self.html_files)
        broken_count = len(self.broken_links)
        
        print(f"\nValidation complete:")
        print(f"  Files checked: {total_files}")
        print(f"  Broken links: {broken_count}")
        
        if broken_count == 0:
            print("  ✅ All links are working correctly!")
            return True
        else:
            print("  ❌ Some links are broken:")
            for source, link, error in self.broken_links[:10]:
                print(f"    {source}: {link} - {error}")
            if broken_count > 10:
                print(f"    ... and {broken_count - 10} more")
            return False
    
    def _find_html_files(self) -> None:
        """Find all HTML files in the book directory."""
        for html_file in self.book_dir.rglob("*.html"):
            if html_file.is_file():
                # Store relative path from book directory
                rel_path = str(html_file.relative_to(self.book_dir))
                self.html_files.add(rel_path)
    
    def _validate_file_links(self, html_file: str) -> None:
        """Validate links in a single HTML file."""
        try:
            file_path = self.book_dir / html_file
            with open(file_path, 'r', encoding='utf-8') as f:
                content = f.read()
            
            # Find all internal links
            link_pattern = re.compile(r'<a[^>]+href="([^"]+)"[^>]*>([^<]*)</a>')
            
            for match in link_pattern.finditer(content):
                href = match.group(1)
                link_text = match.group(2)
                
                # Skip external links
                if self._is_external_link(href):
                    continue
                
                # Validate internal link
                if not self._is_valid_internal_link(html_file, href):
                    self.broken_links.append((
                        html_file,
                        href,
                        f"Target not found: {href}"
                    ))
                    
        except Exception as e:
            self.broken_links.append((
                html_file,
                "N/A",
                f"Error reading file: {e}"
            ))
    
    def _is_external_link(self, href: str) -> bool:
        """Check if a link is external."""
        return (
            href.startswith('http://') or
            href.startswith('https://') or
            href.startswith('mailto:') or
            href.startswith('ftp://')
        )
    
    def _is_valid_internal_link(self, source_file: str, href: str) -> bool:
        """Check if an internal link is valid."""
        # Handle anchor-only links
        if href.startswith('#'):
            # Anchor within the same file - assume valid for now
            # (mdBook handles these automatically)
            return True
        
        # Handle relative links
        if href.startswith('./') or href.startswith('../') or not href.startswith('/'):
            # Resolve relative path
            source_dir = Path(source_file).parent
            target_path = (source_dir / href).resolve()
            
            # Remove anchor if present
            if '#' in str(target_path):
                target_path = Path(str(target_path).split('#')[0])
            
            # Check if target exists
            try:
                rel_target = target_path.relative_to(Path('.'))
                return str(rel_target) in self.html_files
            except ValueError:
                # Target is outside the book directory
                return False
        
        # Handle absolute links within the book
        if href.startswith('/'):
            target_file = href[1:]  # Remove leading slash
            if '#' in target_file:
                target_file = target_file.split('#')[0]
            return target_file in self.html_files
        
        return True  # Default to valid for other cases


def main():
    """Main function to validate built links."""
    validator = BuiltLinkValidator()
    success = validator.validate_all_links()
    
    if success:
        print("\n🎉 All built links are working correctly!")
        exit(0)
    else:
        print("\n⚠️  Some built links are broken.")
        exit(1)


if __name__ == "__main__":
    main()