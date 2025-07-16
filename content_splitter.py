#!/usr/bin/env python3
"""
Content Splitting Script for mdBook Conversion

This script parses the original embedded Rust tutorial markdown document
and extracts content for each section based on header hierarchy, preserving
all formatting, code blocks, tables, and special formatting.
"""

import re
import os
import sys
from pathlib import Path
from typing import Dict, List, Tuple, Optional
from dataclasses import dataclass


@dataclass
class Section:
    """Represents a document section with its metadata and content."""
    title: str
    level: int
    anchor_id: str
    content: str
    subsections: List['Section']
    start_line: int
    end_line: int


class ContentSplitter:
    """Main class for splitting markdown content into mdBook structure."""
    
    def __init__(self, input_file: str, output_dir: str = "src"):
        self.input_file = input_file
        self.output_dir = Path(output_dir)
        self.content = ""
        self.lines = []
        self.sections = []
        
        # Chapter mapping based on the design document
        self.chapter_mapping = {
            "1. Quick Reference": "quick-reference",
            "2. Environment Setup": "environment-setup", 
            "3. Core Language Concepts": "core-concepts",
            "4. Embedded-Specific Patterns": "embedded-patterns",
            "5. Cryptography Implementation": "cryptography",
            "6. Migration and Integration": "migration"
        }
        
        # Subsection filename mapping
        self.subsection_mapping = {
            # Quick Reference subsections
            "1.1 C-to-Rust Syntax Mapping": "syntax-mapping.md",
            "1.2 Memory and Pointer Patterns": "memory-patterns.md", 
            "1.3 Control Flow and Functions": "control-flow.md",
            "1.4 Error Handling Patterns": "error-handling.md",
            "1.5 Crypto-Specific Quick Reference": "crypto-reference.md",
            "1.6 Embedded-Specific Quick Reference": "embedded-reference.md",
            "1.7 Critical Differences and Gotchas": "gotchas.md",
            
            # Environment Setup subsections
            "2.1 Rust Installation and Toolchain": "installation.md",
            "2.2 Target Configuration": "target-config.md",
            "2.3 Project Structure and Dependencies": "project-structure.md",
            "2.4 Build Configuration": "build-config.md", 
            "2.5 Verification and Testing": "verification.md",
            
            # Core Concepts subsections
            "3.1 Ownership and Memory Management": "ownership.md",
            "3.2 Error Handling Without Exceptions": "error-handling.md",
            "3.3 Type System Advantages": "type-system.md",
            "3.4 Advanced Type System Features": "advanced-types.md",
            "3.5 Functional Programming and Data Processing": "functional.md",
            "3.6 Memory Model Differences": "memory-model.md",
            "3.7 Safety Guarantees for Crypto": "safety.md",
            
            # Embedded Patterns subsections
            "4.1 No-std Programming Essentials": "no-std.md",
            "4.2 Hardware Abstraction Patterns": "hardware-abstraction.md",
            "4.3 Interrupt Handling": "interrupts.md",
            "4.4 Static Memory Management": "static-memory.md",
            "4.5 DMA and Hardware Integration": "dma-integration.md",
            
            # Cryptography subsections
            "5.1 Secure Coding Patterns": "secure-patterns.md",
            "5.2 Constant-Time Implementations": "constant-time.md",
            "5.3 Key Management and Zeroization": "key-management.md",
            "5.4 Hardware Crypto Acceleration": "hardware-crypto.md",
            "5.5 Side-Channel Mitigations": "side-channels.md",
            
            # Migration subsections
            "6.1 Incremental Migration Strategies": "strategies.md",
            "6.2 FFI Integration with C Libraries": "ffi-integration.md",
            "6.3 Testing and Validation": "testing.md",
            "6.4 Debugging and Tooling": "debugging.md",
            "6.5 Performance Considerations": "performance.md"
        }
    
    def load_content(self):
        """Load the markdown content from the input file."""
        try:
            with open(self.input_file, 'r', encoding='utf-8') as f:
                self.content = f.read()
                self.lines = self.content.split('\n')
            print(f"✓ Loaded {len(self.lines)} lines from {self.input_file}")
        except FileNotFoundError:
            print(f"✗ Error: Could not find input file {self.input_file}")
            sys.exit(1)
        except Exception as e:
            print(f"✗ Error loading file: {e}")
            sys.exit(1)
    
    def parse_sections(self) -> List[Section]:
        """Parse the document and extract all sections with their hierarchy."""
        sections = []
        current_section = None
        section_stack = []
        
        # Track if we're inside a code block to avoid parsing comments as headers
        in_code_block = False
        
        for line_num, line in enumerate(self.lines):
            # Track code block boundaries
            if line.strip().startswith('```'):
                in_code_block = not in_code_block
                continue
            
            # Skip lines inside code blocks
            if in_code_block:
                continue
            
            # Check for headers at each level
            header_match = None
            level = 0
            
            # Use simple string matching to avoid regex issues
            if line.startswith('#### '):
                level = 4
                title_part = line[5:].strip()
                # Extract anchor if present
                if '{#' in title_part and '}' in title_part:
                    title = title_part.split('{#')[0].strip()
                    anchor_id = title_part.split('{#')[1].split('}')[0]
                else:
                    title = title_part
                    anchor_id = self._generate_anchor_id(title)
                header_match = True
            elif line.startswith('### '):
                level = 3
                title_part = line[4:].strip()
                # Extract anchor if present
                if '{#' in title_part and '}' in title_part:
                    title = title_part.split('{#')[0].strip()
                    anchor_id = title_part.split('{#')[1].split('}')[0]
                else:
                    title = title_part
                    anchor_id = self._generate_anchor_id(title)
                header_match = True
            elif line.startswith('## '):
                level = 2
                title_part = line[3:].strip()
                # Extract anchor if present
                if '{#' in title_part and '}' in title_part:
                    title = title_part.split('{#')[0].strip()
                    anchor_id = title_part.split('{#')[1].split('}')[0]
                else:
                    title = title_part
                    anchor_id = self._generate_anchor_id(title)
                header_match = True
            elif line.startswith('# '):
                level = 1
                title_part = line[2:].strip()
                # Extract anchor if present
                if '{#' in title_part and '}' in title_part:
                    title = title_part.split('{#')[0].strip()
                    anchor_id = title_part.split('{#')[1].split('}')[0]
                else:
                    title = title_part
                    anchor_id = self._generate_anchor_id(title)
                header_match = True
            
            if header_match and level > 0:
                # Close previous section if exists
                if current_section:
                    current_section.end_line = line_num - 1
                
                # Create new section
                section = Section(
                    title=title,
                    level=level,
                    anchor_id=anchor_id,
                    content="",
                    subsections=[],
                    start_line=line_num,
                    end_line=len(self.lines) - 1  # Will be updated when next section starts
                )
                
                # Handle section hierarchy
                while section_stack and section_stack[-1].level >= level:
                    section_stack.pop()
                
                if section_stack:
                    section_stack[-1].subsections.append(section)
                else:
                    sections.append(section)
                
                section_stack.append(section)
                current_section = section
        
        # Close the last section
        if current_section:
            current_section.end_line = len(self.lines) - 1
        
        # Extract content for each section
        self._extract_section_content(sections)
        
        return sections
    
    def _generate_anchor_id(self, title: str) -> str:
        """Generate an anchor ID from a title."""
        # Remove special characters and convert to lowercase
        anchor = re.sub(r'[^\w\s-]', '', title.lower())
        anchor = re.sub(r'[-\s]+', '-', anchor)
        return anchor.strip('-')
    
    def _extract_section_content(self, sections: List[Section]):
        """Extract content for each section recursively."""
        for i, section in enumerate(sections):
            # Determine content boundaries
            start_line = section.start_line + 1  # Skip the header line
            
            # Find the end line (start of next section at same level)
            if i + 1 < len(sections):
                # Next section at same level
                end_line = sections[i + 1].start_line - 1
            else:
                # Last section at this level
                end_line = section.end_line
            
            # Extract content including subsections
            section.content = '\n'.join(self.lines[start_line:end_line + 1]).strip()
            
            # Recursively extract subsection content
            self._extract_section_content(section.subsections)
    
    def create_directory_structure(self):
        """Create the mdBook directory structure."""
        # Create main src directory
        self.output_dir.mkdir(exist_ok=True)
        
        # Create chapter directories
        for chapter_name, chapter_dir in self.chapter_mapping.items():
            chapter_path = self.output_dir / chapter_dir
            chapter_path.mkdir(exist_ok=True)
            print(f"✓ Created directory: {chapter_path}")
    
    def split_content(self):
        """Main method to split content into mdBook structure."""
        print("🚀 Starting content splitting...")
        
        # Load and parse content
        self.load_content()
        sections = self.parse_sections()
        
        # Create directory structure
        self.create_directory_structure()
        
        # Process each major section (chapters)
        # Look for level 2 sections in the subsections of the main document
        for section in sections:
            print(f"Checking section: '{section.title}' (level {section.level})")
            if section.level == 1:  # Main document section
                for subsection in section.subsections:
                    print(f"  Checking subsection: '{subsection.title}' (level {subsection.level})")
                    if subsection.level == 2 and subsection.title in self.chapter_mapping:
                        print(f"  Processing chapter: {subsection.title}")
                        self._process_chapter(subsection)
            elif section.level == 2 and section.title in self.chapter_mapping:
                print(f"Processing chapter: {section.title}")
                self._process_chapter(section)
        
        # Create introduction file
        self._create_introduction()
        
        # Create SUMMARY.md
        self._create_summary(sections)
        
        print("✅ Content splitting completed successfully!")
    
    def _process_chapter(self, chapter_section: Section):
        """Process a chapter and its subsections."""
        # Find chapter directory
        chapter_dir = self.chapter_mapping.get(chapter_section.title)
        
        if not chapter_dir:
            print(f"⚠️  Warning: No mapping found for chapter: {chapter_section.title}")
            return
        
        chapter_path = self.output_dir / chapter_dir
        
        # Create chapter README.md (overview)
        readme_content = self._create_chapter_overview(chapter_section)
        readme_path = chapter_path / "README.md"
        with open(readme_path, 'w', encoding='utf-8') as f:
            f.write(readme_content)
        print(f"✓ Created: {readme_path}")
        
        # Process subsections
        for subsection in chapter_section.subsections:
            self._process_subsection(subsection, chapter_path)
    
    def _process_subsection(self, subsection: Section, chapter_path: Path):
        """Process a subsection and create its markdown file."""
        # Find filename mapping
        filename = None
        for key, mapped_filename in self.subsection_mapping.items():
            if subsection.title in key or key.endswith(subsection.title):
                filename = mapped_filename
                break
        
        if not filename:
            # Generate filename from title
            filename = self._generate_filename(subsection.title)
        
        # Create subsection file
        subsection_path = chapter_path / filename
        
        # Process content to handle cross-references and formatting
        processed_content = self._process_content(subsection.content, subsection.title)
        
        with open(subsection_path, 'w', encoding='utf-8') as f:
            f.write(f"# {subsection.title}\n\n{processed_content}")
        
        print(f"✓ Created: {subsection_path}")
    
    def _generate_filename(self, title: str) -> str:
        """Generate a filename from a section title."""
        # Remove section numbers and special characters
        clean_title = re.sub(r'^\d+\.\d*\s*', '', title)
        filename = re.sub(r'[^\w\s-]', '', clean_title.lower())
        filename = re.sub(r'[-\s]+', '-', filename)
        return f"{filename.strip('-')}.md"
    
    def _create_chapter_overview(self, chapter_section: Section) -> str:
        """Create overview content for a chapter README.md."""
        title = chapter_section.title
        
        # Extract the first paragraph or description from the chapter content
        content_lines = chapter_section.content.split('\n')
        description = ""
        
        # Find the first substantial paragraph
        for line in content_lines:
            line = line.strip()
            if line and not line.startswith('#') and not line.startswith('|') and not line.startswith('-'):
                if len(line) > 50:  # Substantial content
                    description = line
                    break
        
        # Create overview content
        overview = f"# {title}\n\n"
        
        if description:
            overview += f"{description}\n\n"
        
        # Add subsection navigation
        if chapter_section.subsections:
            overview += "## Sections\n\n"
            for subsection in chapter_section.subsections:
                filename = None
                for key, mapped_filename in self.subsection_mapping.items():
                    if subsection.title in key or key.endswith(subsection.title):
                        filename = mapped_filename
                        break
                
                if not filename:
                    filename = self._generate_filename(subsection.title)
                
                overview += f"- [{subsection.title}](./{filename})\n"
        
        return overview
    
    def _process_content(self, content: str, section_title: str) -> str:
        """Process content to handle cross-references, expandable sections, and formatting."""
        if not content:
            return ""
        
        # Handle expandable sections (details/summary)
        content = self._process_expandable_sections(content)
        
        # Handle cross-references
        content = self._process_cross_references(content)
        
        # Handle code blocks (ensure they're preserved)
        content = self._process_code_blocks(content)
        
        # Handle tables (ensure they're preserved)
        content = self._process_tables(content)
        
        return content
    
    def _process_expandable_sections(self, content: str) -> str:
        """Process expandable sections (details/summary tags)."""
        # These are already in proper HTML format for mdBook, so preserve them
        return content
    
    def _process_cross_references(self, content: str) -> str:
        """Process cross-references to update links for mdBook structure."""
        # Pattern to match internal links like [text](#anchor) or [text](section-link)
        link_pattern = re.compile(r'\[([^\]]+)\]\(#([^)]+)\)')
        
        def replace_link(match):
            link_text = match.group(1)
            anchor = match.group(2)
            
            # Try to map anchor to new mdBook structure
            new_link = self._map_anchor_to_mdbook_link(anchor)
            if new_link:
                return f"[{link_text}]({new_link})"
            else:
                # Keep original if no mapping found
                return match.group(0)
        
        return link_pattern.sub(replace_link, content)
    
    def _map_anchor_to_mdbook_link(self, anchor: str) -> Optional[str]:
        """Map original anchors to mdBook relative links."""
        # This is a simplified mapping - in a full implementation,
        # you'd want a comprehensive mapping table
        anchor_mappings = {
            'quick-reference': '../quick-reference/README.md',
            'environment-setup': '../environment-setup/README.md',
            'core-language-concepts': '../core-concepts/README.md',
            'embedded-specific-patterns': '../embedded-patterns/README.md',
            'cryptography-implementation': '../cryptography/README.md',
            'migration-and-integration': '../migration/README.md',
            
            # Add more specific mappings as needed
            'c-to-rust-syntax-mapping': '../quick-reference/syntax-mapping.md',
            'memory-and-pointer-patterns': '../quick-reference/memory-patterns.md',
            'ownership-and-memory-management': '../core-concepts/ownership.md',
            'error-handling-without-exceptions': '../core-concepts/error-handling.md',
        }
        
        return anchor_mappings.get(anchor)
    
    def _process_code_blocks(self, content: str) -> str:
        """Ensure code blocks are properly preserved."""
        # Code blocks should already be in proper markdown format
        # Just ensure they're preserved as-is
        return content
    
    def _process_tables(self, content: str) -> str:
        """Ensure tables are properly preserved."""
        # Tables should already be in proper markdown format
        # Just ensure they're preserved as-is
        return content
    
    def _create_introduction(self):
        """Create the introduction.md file."""
        # Extract introduction content from the original document
        intro_content = ""
        in_intro = False
        
        for line in self.lines:
            if line.strip() == "## Introduction":
                in_intro = True
                continue
            elif in_intro and line.startswith('## '):
                break
            elif in_intro:
                intro_content += line + '\n'
        
        # Create introduction file
        intro_path = self.output_dir / "introduction.md"
        with open(intro_path, 'w', encoding='utf-8') as f:
            f.write("# Introduction\n\n" + intro_content.strip())
        
        print(f"✓ Created: {intro_path}")
    
    def _create_summary(self, sections: List[Section]):
        """Create the SUMMARY.md file for mdBook table of contents."""
        summary_content = "# Summary\n\n"
        summary_content += "[Introduction](introduction.md)\n\n"
        
        # Add chapters - look in subsections of main document
        for section in sections:
            if section.level == 1:  # Main document section
                for subsection in section.subsections:
                    if subsection.level == 2 and subsection.title in self.chapter_mapping:
                        chapter_dir = self.chapter_mapping[subsection.title]
                        summary_content += f"- [{subsection.title}]({chapter_dir}/README.md)\n"
                        
                        # Add subsections
                        for subsubsection in subsection.subsections:
                            filename = None
                            for key, mapped_filename in self.subsection_mapping.items():
                                if subsubsection.title in key or key.endswith(subsubsection.title):
                                    filename = mapped_filename
                                    break
                            
                            if not filename:
                                filename = self._generate_filename(subsubsection.title)
                            
                            summary_content += f"  - [{subsubsection.title}]({chapter_dir}/{filename})\n"
                        
                        summary_content += "\n"
            elif section.level == 2 and section.title in self.chapter_mapping:
                chapter_dir = self.chapter_mapping[section.title]
                summary_content += f"- [{section.title}]({chapter_dir}/README.md)\n"
                
                # Add subsections
                for subsection in section.subsections:
                    filename = None
                    for key, mapped_filename in self.subsection_mapping.items():
                        if subsection.title in key or key.endswith(subsection.title):
                            filename = mapped_filename
                            break
                    
                    if not filename:
                        filename = self._generate_filename(subsection.title)
                    
                    summary_content += f"  - [{subsection.title}]({chapter_dir}/{filename})\n"
                
                summary_content += "\n"
        
        # Create SUMMARY.md file
        summary_path = self.output_dir / "SUMMARY.md"
        with open(summary_path, 'w', encoding='utf-8') as f:
            f.write(summary_content)
        
        print(f"✓ Created: {summary_path}")


def main():
    """Main function to run the content splitter."""
    if len(sys.argv) < 2:
        print("Usage: python content_splitter.py <input_markdown_file> [output_directory]")
        sys.exit(1)
    
    input_file = sys.argv[1]
    output_dir = sys.argv[2] if len(sys.argv) > 2 else "src"
    
    splitter = ContentSplitter(input_file, output_dir)
    splitter.split_content()


if __name__ == "__main__":
    main()