#!/usr/bin/env python3
"""
Built mdBook Link Validator

This script validates links in the built mdBook HTML files to ensure
all internal references work correctly after the build process.
"""

import os
import re
import json
from pathlib import Path
from typing import Dict, List, Set
from urllib.parse import urljoin, urlparse
import subprocess


class BuiltLinkValidator:
    """Validates links in built mdBook HTML files"""
    
    def __init__(self, book_dir: str = "book"):
        self.book_dir = Path(book_dir)
        self.html_files = []
        self.broken_links = []
        self.all_links = []
        
    def scan_html_files(self) -> None:
        """Scan all HTML files in the book directory"""
        if not self.book_dir.exists():
            print(f"Building mdBook first...")
            self._build_book()
        
        self.html_files = list(self.book_dir.rglob("*.html"))
        print(f"Found {len(self.html_files)} HTML files")
    
    def _build_book(self) -> bool:
        """Build the mdBook"""
        try:
            result = subprocess.run(['mdbook', 'build'], capture_output=True, text=True)
            return result.returncode == 0
        except:
            return False
    
    def validate_links(self) -> Dict:
        """Validate all links in HTML files"""
        print("Validating links in built HTML...")
        
        # Extract all links
        for html_file in self.html_files:
            self._extract_links_from_html(html_file)
        
        # Validate each link
        for link in self.all_links:
            if not self._validate_link(link):
                self.broken_links.append(link)
        
        return {
            'total_links': len(self.all_links),
            'broken_links': len(self.broken_links),
            'success_rate': (len(self.all_links) - len(self.broken_links)) / max(len(self.all_links), 1)
        }
    
    def _extract_links_from_html(self, html_file: Path) -> None:
        """Extract links from HTML file"""
        try:
            with open(html_file, 'r', encoding='utf-8') as f:
                content = f.read()
            
            # Find all href attributes
            href_pattern = r'href=["\']([^"\']+)["\']'
            matches = re.findall(href_pattern, content)
            
            for href in matches:
                if not self._is_external_link(href) and not href.startswith('#'):
                    self.all_links.append({
                        'source': str(html_file.relative_to(self.book_dir)),
                        'target': href,
                        'type': 'internal'
                    })
        
        except Exception as e:
            print(f"Error reading {html_file}: {e}")
    
    def _is_external_link(self, href: str) -> bool:
        """Check if link is external"""
        return href.startswith(('http://', 'https://', 'mailto:', 'ftp://'))
    
    def _validate_link(self, link: Dict) -> bool:
        """Validate a single link"""
        target = link['target']
        source_file = self.book_dir / link['source']
        
        # Resolve relative path
        if target.startswith('./'):
            target_path = source_file.parent / target[2:]
        elif target.startswith('../'):
            target_path = source_file.parent / target
        else:
            target_path = self.book_dir / target
        
        # Remove anchor
        if '#' in str(target_path):
            target_path = Path(str(target_path).split('#')[0])
        
        return target_path.exists()
    
    def generate_report(self) -> str:
        """Generate validation report"""
        stats = self.validate_links()
        
        report = f"""# Built mdBook Link Validation Report

**Generated:** {__import__('datetime').datetime.now().strftime('%Y-%m-%d %H:%M:%S')}

## Summary

- **Total Links:** {stats['total_links']}
- **Broken Links:** {stats['broken_links']}
- **Success Rate:** {stats['success_rate']:.1%}

## Broken Links

"""
        
        if self.broken_links:
            for link in self.broken_links:
                report += f"- `{link['source']}` → `{link['target']}`\n"
        else:
            report += "✅ No broken links found!\n"
        
        return report


def main():
    """Main function"""
    validator = BuiltLinkValidator()
    validator.scan_html_files()
    
    report = validator.generate_report()
    
    with open("built_link_validation_report.md", 'w') as f:
        f.write(report)
    
    print("Built link validation complete!")
    print(f"Report saved to: built_link_validation_report.md")


if __name__ == "__main__":
    main()