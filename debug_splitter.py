#!/usr/bin/env python3
"""Debug script to check section parsing."""

import re
from content_splitter import ContentSplitter

def debug_sections():
    splitter = ContentSplitter("embedded-rust-tutorial-master.md", "src")
    splitter.load_content()
    sections = splitter.parse_sections()
    
    print(f"Found {len(sections)} top-level sections:")
    for i, section in enumerate(sections):
        print(f"{i+1}. Level {section.level}: '{section.title}' (lines {section.start_line}-{section.end_line})")
        print(f"   Anchor: {section.anchor_id}")
        print(f"   Subsections: {len(section.subsections)}")
        for j, subsection in enumerate(section.subsections):
            print(f"     {j+1}. Level {subsection.level}: '{subsection.title}'")
        print()

if __name__ == "__main__":
    debug_sections()