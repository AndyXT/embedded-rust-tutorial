#!/usr/bin/env python3
"""
Link Validation and Cross-Reference System for mdBook Conversion

This script validates all internal links in the mdBook structure and updates
cross-references to use mdBook relative linking conventions.
"""

import os
import re
import json
from pathlib import Path
from typing import Dict, List, Set, Tuple, Optional
from dataclasses import dataclass
from urllib.parse import urlparse, unquote


@dataclass
class LinkInfo:
    """Information about a link found in markdown files."""
    source_file: str
    line_number: int
    link_text: str
    link_target: str
    anchor: Optional[str] = None
    is_valid: bool = True
    error_message: str = ""


@dataclass
class FileInfo:
    """Information about a markdown file."""
    path: str
    title: str
    headers: List[str]
    links: List[LinkInfo]


class LinkValidator:
    """Validates and updates links in mdBook structure."""
    
    def __init__(self, src_dir: str = "src"):
        self.src_dir = Path(src_dir)
        self.files: Dict[str, FileInfo] = {}
        self.all_links: List[LinkInfo] = []
        self.broken_links: List[LinkInfo] = []
        self.updated_links: List[Tuple[str, str, str]] = []  # (file, old_link, new_link)
        
        # mdBook specific patterns
        self.mdbook_link_pattern = re.compile(r'\[([^\]]+)\]\(([^)]+)\)')
        self.header_pattern = re.compile(r'^(#{1,6})\s+(.+)$', re.MULTILINE)
        self.anchor_pattern = re.compile(r'#([a-zA-Z0-9_-]+)')
        
    def scan_all_files(self) -> None:
        """Scan all markdown files in the src directory."""
        print("Scanning markdown files...")
        
        for md_file in self.src_dir.rglob("*.md"):
            if md_file.is_file():
                self._scan_file(md_file)
        
        print(f"Scanned {len(self.files)} files")
        print(f"Found {len(self.all_links)} total links")
    
    def _scan_file(self, file_path: Path) -> None:
        """Scan a single markdown file for links and headers."""
        try:
            with open(file_path, 'r', encoding='utf-8') as f:
                content = f.read()
            
            relative_path = str(file_path.relative_to(self.src_dir))
            
            # Extract title (first h1 header)
            title = self._extract_title(content)
            
            # Extract all headers for anchor validation
            headers = self._extract_headers(content)
            
            # Extract all links
            links = self._extract_links(content, relative_path)
            
            file_info = FileInfo(
                path=relative_path,
                title=title,
                headers=headers,
                links=links
            )
            
            self.files[relative_path] = file_info
            self.all_links.extend(links)
            
        except Exception as e:
            print(f"Error scanning {file_path}: {e}")
    
    def _extract_title(self, content: str) -> str:
        """Extract the title from the first h1 header."""
        match = re.search(r'^#\s+(.+)$', content, re.MULTILINE)
        return match.group(1) if match else "Untitled"
    
    def _extract_headers(self, content: str) -> List[str]:
        """Extract all headers from the content."""
        headers = []
        for match in self.header_pattern.finditer(content):
            header_text = match.group(2).strip()
            # Convert to anchor format (lowercase, spaces to hyphens, remove special chars)
            anchor = self._text_to_anchor(header_text)
            headers.append(anchor)
        return headers
    
    def _text_to_anchor(self, text: str) -> str:
        """Convert header text to anchor format."""
        # Remove markdown formatting
        text = re.sub(r'[*_`]', '', text)
        # Convert to lowercase and replace spaces with hyphens
        anchor = re.sub(r'[^\w\s-]', '', text.lower())
        anchor = re.sub(r'[-\s]+', '-', anchor)
        return anchor.strip('-')
    
    def _extract_links(self, content: str, source_file: str) -> List[LinkInfo]:
        """Extract all markdown links from content."""
        links = []
        
        for line_num, line in enumerate(content.split('\n'), 1):
            for match in self.mdbook_link_pattern.finditer(line):
                link_text = match.group(1)
                link_target = match.group(2)
                
                # Skip external links
                if self._is_external_link(link_target):
                    continue
                
                # Parse anchor if present
                anchor = None
                if '#' in link_target:
                    link_target, anchor = link_target.split('#', 1)
                
                link_info = LinkInfo(
                    source_file=source_file,
                    line_number=line_num,
                    link_text=link_text,
                    link_target=link_target,
                    anchor=anchor
                )
                
                links.append(link_info)
        
        return links
    
    def _is_external_link(self, link: str) -> bool:
        """Check if a link is external (http/https/mailto/etc)."""
        parsed = urlparse(link)
        return bool(parsed.scheme)
    
    def validate_all_links(self) -> None:
        """Validate all internal links."""
        print("Validating links...")
        
        for link in self.all_links:
            self._validate_link(link)
        
        broken_count = len(self.broken_links)
        total_count = len(self.all_links)
        
        print(f"Validation complete: {total_count - broken_count}/{total_count} links valid")
        if broken_count > 0:
            print(f"Found {broken_count} broken links")
    
    def _validate_link(self, link: LinkInfo) -> None:
        """Validate a single link."""
        # Handle empty links (anchor-only)
        if not link.link_target.strip():
            if link.anchor:
                # Self-reference anchor - check if anchor exists in current file
                source_file_info = self.files.get(link.source_file)
                if source_file_info and link.anchor not in source_file_info.headers:
                    link.is_valid = False
                    link.error_message = f"Anchor #{link.anchor} not found in current file"
                    self.broken_links.append(link)
            return
        
        # Resolve relative path
        target_path = self._resolve_relative_path(link.source_file, link.link_target)
        
        # Check if target file exists
        if target_path not in self.files:
            link.is_valid = False
            link.error_message = f"Target file {target_path} not found"
            self.broken_links.append(link)
            return
        
        # Check anchor if present
        if link.anchor:
            target_file_info = self.files[target_path]
            if link.anchor not in target_file_info.headers:
                link.is_valid = False
                link.error_message = f"Anchor #{link.anchor} not found in {target_path}"
                self.broken_links.append(link)
    
    def _resolve_relative_path(self, source_file: str, target_link: str) -> str:
        """Resolve relative path from source file to target."""
        # Normalize path separators
        source_file = source_file.replace('\\', '/')
        target_link = target_link.replace('\\', '/')
        
        source_dir = str(Path(source_file).parent)
        if source_dir == '.':
            source_dir = ''
        
        # Handle different link formats
        if target_link.startswith('./'):
            # Same directory reference
            if source_dir:
                result = f"{source_dir}/{target_link[2:]}"
            else:
                result = target_link[2:]
        elif target_link.startswith('../'):
            # Parent directory reference
            if source_dir:
                # Go up one level from source directory
                parent_parts = source_dir.split('/')
                if len(parent_parts) > 0:
                    parent_parts.pop()
                parent_dir = '/'.join(parent_parts) if parent_parts else ''
                remaining_link = target_link[3:]  # Remove '../'
                
                # Handle multiple levels of '../'
                while remaining_link.startswith('../'):
                    if parent_parts:
                        parent_parts.pop()
                    parent_dir = '/'.join(parent_parts) if parent_parts else ''
                    remaining_link = remaining_link[3:]
                
                if parent_dir:
                    result = f"{parent_dir}/{remaining_link}"
                else:
                    result = remaining_link
            else:
                result = target_link[3:]  # Remove '../' from root
        else:
            # Direct reference (relative to source directory)
            if source_dir:
                result = f"{source_dir}/{target_link}"
            else:
                result = target_link
        
        return result.replace('\\', '/')
    
    def update_cross_references(self) -> None:
        """Update cross-references to use mdBook conventions."""
        print("Updating cross-references...")
        
        updates_made = 0
        
        for file_path, file_info in self.files.items():
            file_updated = False
            
            try:
                with open(self.src_dir / file_path, 'r', encoding='utf-8') as f:
                    content = f.read()
                
                original_content = content
                
                # Update links in this file
                for link in file_info.links:
                    if not link.is_valid:
                        continue
                    
                    old_link_full = link.link_target + ('#' + link.anchor if link.anchor else '')
                    new_link = self._generate_mdbook_link(file_path, link)
                    
                    if new_link != old_link_full:
                        # Create regex pattern to match the exact link
                        escaped_text = re.escape(link.link_text)
                        escaped_target = re.escape(old_link_full)
                        pattern = f'\\[{escaped_text}\\]\\({escaped_target}\\)'
                        replacement = f'[{link.link_text}]({new_link})'
                        
                        if re.search(pattern, content):
                            content = re.sub(pattern, replacement, content)
                            self.updated_links.append((file_path, f'[{link.link_text}]({old_link_full})', replacement))
                            file_updated = True
                
                # Write updated content if changes were made
                if file_updated and content != original_content:
                    with open(self.src_dir / file_path, 'w', encoding='utf-8') as f:
                        f.write(content)
                    updates_made += 1
                    
            except Exception as e:
                print(f"Error updating {file_path}: {e}")
        
        print(f"Updated {updates_made} files with {len(self.updated_links)} link changes")
    
    def _generate_mdbook_link(self, source_file: str, link: LinkInfo) -> str:
        """Generate proper mdBook link format."""
        if not link.link_target.strip():
            # Self-reference anchor
            return f"#{link.anchor}" if link.anchor else ""
        
        # For mdBook, we want relative paths from the source file
        target_path = link.link_target
        
        # Ensure .md extension if it's a file reference
        if not target_path.endswith('.md') and not target_path.endswith('/') and '.' not in Path(target_path).name:
            target_path += '.md'
        
        # Build the link
        result = target_path
        if link.anchor:
            result += f"#{link.anchor}"
        
        return result
    
    def fix_common_link_patterns(self) -> None:
        """Fix common link patterns that need updating for mdBook."""
        print("Fixing common link patterns...")
        
        fixes_applied = 0
        
        for file_path, file_info in self.files.items():
            try:
                with open(self.src_dir / file_path, 'r', encoding='utf-8') as f:
                    content = f.read()
                
                original_content = content
                
                # Fix README.md links to use proper chapter references
                content = re.sub(
                    r'\[([^\]]+)\]\(\.\/README\.md\)',
                    r'[\1](./README.md)',
                    content
                )
                
                # Fix relative links that should be absolute within the book
                content = re.sub(
                    r'\[([^\]]+)\]\(\.\.\/([^)]+)\)',
                    lambda m: f"[{m.group(1)}]({self._fix_parent_link(file_path, m.group(2))})",
                    content
                )
                
                # Fix anchor-only links to be more explicit
                content = re.sub(
                    r'\[([^\]]+)\]\(#([^)]+)\)',
                    r'[\1](#\2)',
                    content
                )
                
                if content != original_content:
                    with open(self.src_dir / file_path, 'w', encoding='utf-8') as f:
                        f.write(content)
                    fixes_applied += 1
                    
            except Exception as e:
                print(f"Error fixing patterns in {file_path}: {e}")
        
        print(f"Applied pattern fixes to {fixes_applied} files")
    
    def _fix_parent_link(self, source_file: str, target_link: str) -> str:
        """Fix parent directory links to work with mdBook structure."""
        source_parts = Path(source_file).parts
        
        # If we're in a subdirectory, adjust the link
        if len(source_parts) > 1:
            # Remove one level of parent directory reference
            if target_link.startswith('../'):
                target_link = target_link[3:]
            return f"../{target_link}"
        
        return target_link
    
    def generate_report(self) -> Dict:
        """Generate a comprehensive validation report."""
        report = {
            "summary": {
                "total_files": len(self.files),
                "total_links": len(self.all_links),
                "broken_links": len(self.broken_links),
                "updated_links": len(self.updated_links),
                "validation_success_rate": (len(self.all_links) - len(self.broken_links)) / len(self.all_links) * 100 if self.all_links else 100
            },
            "broken_links": [
                {
                    "source_file": link.source_file,
                    "line_number": link.line_number,
                    "link_text": link.link_text,
                    "link_target": link.link_target,
                    "anchor": link.anchor,
                    "error": link.error_message
                }
                for link in self.broken_links
            ],
            "updated_links": [
                {
                    "file": file_path,
                    "old_link": old_link,
                    "new_link": new_link
                }
                for file_path, old_link, new_link in self.updated_links
            ],
            "file_statistics": {
                file_path: {
                    "title": file_info.title,
                    "link_count": len(file_info.links),
                    "header_count": len(file_info.headers),
                    "broken_link_count": len([l for l in file_info.links if not l.is_valid])
                }
                for file_path, file_info in self.files.items()
            }
        }
        
        return report
    
    def save_report(self, filename: str = "link_validation_report.json") -> None:
        """Save validation report to JSON file."""
        report = self.generate_report()
        
        with open(filename, 'w', encoding='utf-8') as f:
            json.dump(report, f, indent=2, ensure_ascii=False)
        
        print(f"Validation report saved to {filename}")
    
    def print_summary(self) -> None:
        """Print a summary of the validation results."""
        report = self.generate_report()
        summary = report["summary"]
        
        print("\n" + "="*60)
        print("LINK VALIDATION SUMMARY")
        print("="*60)
        print(f"Total files scanned: {summary['total_files']}")
        print(f"Total links found: {summary['total_links']}")
        print(f"Broken links: {summary['broken_links']}")
        print(f"Links updated: {summary['updated_links']}")
        print(f"Success rate: {summary['validation_success_rate']:.1f}%")
        
        if self.broken_links:
            print("\nBROKEN LINKS:")
            print("-" * 40)
            for link in self.broken_links[:10]:  # Show first 10
                print(f"  {link.source_file}:{link.line_number}")
                print(f"    [{link.link_text}]({link.link_target}{'#' + link.anchor if link.anchor else ''})")
                print(f"    Error: {link.error_message}")
                print()
            
            if len(self.broken_links) > 10:
                print(f"  ... and {len(self.broken_links) - 10} more")
        
        print("="*60)


def main():
    """Main function to run link validation."""
    validator = LinkValidator()
    
    # Scan all files
    validator.scan_all_files()
    
    # Validate links
    validator.validate_all_links()
    
    # Fix common patterns
    validator.fix_common_link_patterns()
    
    # Update cross-references
    validator.update_cross_references()
    
    # Re-validate after updates
    print("\nRe-validating after updates...")
    validator.all_links.clear()
    validator.broken_links.clear()
    validator.files.clear()
    
    validator.scan_all_files()
    validator.validate_all_links()
    
    # Generate and save report
    validator.save_report()
    
    # Print summary
    validator.print_summary()


if __name__ == "__main__":
    main()