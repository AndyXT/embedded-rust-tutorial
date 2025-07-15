#!/usr/bin/env python3
"""
Content Analysis Script for Embedded Rust Tutorial
Identifies duplicate sections, redundant information, and maps content to proposed structure.
"""

import re
import hashlib
from collections import defaultdict, Counter
from dataclasses import dataclass
from typing import List, Dict, Set, Tuple
import json

@dataclass
class Section:
    title: str
    level: int
    content: str
    line_start: int
    line_end: int
    word_count: int
    code_blocks: List[str]
    
@dataclass
class DuplicateContent:
    content: str
    sections: List[str]
    similarity_score: float
    
@dataclass
class ContentMapping:
    original_section: str
    proposed_section: str
    action: str  # 'keep', 'merge', 'consolidate', 'eliminate'
    reason: str

class ContentAnalyzer:
    def __init__(self, file_path: str):
        self.file_path = file_path
        self.content = self._read_file()
        self.sections = self._parse_sections()
        self.duplicates = []
        self.mappings = []
        
    def _read_file(self) -> str:
        """Read the tutorial file content."""
        with open(self.file_path, 'r', encoding='utf-8') as f:
            return f.read()
    
    def _parse_sections(self) -> List[Section]:
        """Parse the document into sections."""
        sections = []
        lines = self.content.split('\n')
        current_section = None
        current_content = []
        
        for i, line in enumerate(lines):
            # Check for markdown headers
            header_match = re.match(r'^(#{1,6})\s+(.+?)(?:\s*\{#[\w-]+\})?\s*$', line)
            
            if header_match:
                # Save previous section
                if current_section:
                    content_text = '\n'.join(current_content)
                    code_blocks = self._extract_code_blocks(content_text)
                    sections.append(Section(
                        title=current_section['title'],
                        level=current_section['level'],
                        content=content_text,
                        line_start=current_section['start'],
                        line_end=i,
                        word_count=len(content_text.split()),
                        code_blocks=code_blocks
                    ))
                
                # Start new section
                level = len(header_match.group(1))
                title = header_match.group(2).strip()
                current_section = {
                    'title': title,
                    'level': level,
                    'start': i
                }
                current_content = []
            else:
                if current_section:
                    current_content.append(line)
        
        # Add final section
        if current_section:
            content_text = '\n'.join(current_content)
            code_blocks = self._extract_code_blocks(content_text)
            sections.append(Section(
                title=current_section['title'],
                level=current_section['level'],
                content=content_text,
                line_start=current_section['start'],
                line_end=len(lines),
                word_count=len(content_text.split()),
                code_blocks=code_blocks
            ))
        
        return sections
    
    def _extract_code_blocks(self, content: str) -> List[str]:
        """Extract code blocks from content."""
        code_blocks = []
        # Find fenced code blocks
        pattern = r'```[\w]*\n(.*?)\n```'
        matches = re.findall(pattern, content, re.DOTALL)
        code_blocks.extend(matches)
        
        # Find inline code
        inline_pattern = r'`([^`]+)`'
        inline_matches = re.findall(inline_pattern, content)
        code_blocks.extend(inline_matches)
        
        return code_blocks
    
    def analyze_duplicates(self, similarity_threshold: float = 0.7) -> List[DuplicateContent]:
        """Identify duplicate or highly similar content."""
        duplicates = []
        
        # Compare sections pairwise
        for i, section1 in enumerate(self.sections):
            for j, section2 in enumerate(self.sections[i+1:], i+1):
                similarity = self._calculate_similarity(section1.content, section2.content)
                
                if similarity > similarity_threshold:
                    duplicates.append(DuplicateContent(
                        content=self._get_common_content(section1.content, section2.content),
                        sections=[section1.title, section2.title],
                        similarity_score=similarity
                    ))
        
        # Group similar duplicates
        self.duplicates = self._group_duplicates(duplicates)
        return self.duplicates
    
    def _calculate_similarity(self, text1: str, text2: str) -> float:
        """Calculate similarity between two text blocks."""
        # Simple word-based similarity
        words1 = set(re.findall(r'\w+', text1.lower()))
        words2 = set(re.findall(r'\w+', text2.lower()))
        
        if not words1 or not words2:
            return 0.0
        
        intersection = words1.intersection(words2)
        union = words1.union(words2)
        
        return len(intersection) / len(union)
    
    def _get_common_content(self, text1: str, text2: str) -> str:
        """Extract common content between two texts."""
        # Simple implementation - return first 200 chars of longer text
        longer_text = text1 if len(text1) > len(text2) else text2
        return longer_text[:200] + "..." if len(longer_text) > 200 else longer_text
    
    def _group_duplicates(self, duplicates: List[DuplicateContent]) -> List[DuplicateContent]:
        """Group related duplicates together."""
        # Simple grouping - return as is for now
        return duplicates
    
    def identify_redundant_patterns(self) -> Dict[str, List[str]]:
        """Identify common redundant patterns across sections."""
        patterns = {
            'setup_instructions': [],
            'memory_management_explanations': [],
            'error_handling_patterns': [],
            'code_examples': [],
            'c_to_rust_comparisons': []
        }
        
        for section in self.sections:
            content_lower = section.content.lower()
            
            # Check for setup-related content
            if any(keyword in content_lower for keyword in ['install', 'setup', 'cargo', 'rustup']):
                patterns['setup_instructions'].append(section.title)
            
            # Check for memory management explanations
            if any(keyword in content_lower for keyword in ['ownership', 'borrowing', 'memory', 'malloc', 'free']):
                patterns['memory_management_explanations'].append(section.title)
            
            # Check for error handling
            if any(keyword in content_lower for keyword in ['error', 'result', 'option', 'unwrap', 'expect']):
                patterns['error_handling_patterns'].append(section.title)
            
            # Check for C to Rust comparisons
            if 'c pattern' in content_lower or 'rust equivalent' in content_lower or '|' in section.content:
                patterns['c_to_rust_comparisons'].append(section.title)
        
        return patterns
    
    def generate_content_mapping(self) -> List[ContentMapping]:
        """Generate mapping from current structure to proposed structure."""
        proposed_structure = {
            'Quick Reference Section': [
                'Quick Reference: C to Rust Cheat Sheet',
                'Common Patterns and Best Practices'
            ],
            'Environment Setup Section': [
                'Setting Up Your Environment'
            ],
            'Core Language Concepts Section': [
                'Core Language Differences from C',
                'Ownership and Memory Management',
                'Error Handling Without Exceptions'
            ],
            'Embedded-Specific Patterns Section': [
                'No-std Programming',
                'Working with Hardware',
                'Project Structure and Organization for Embedded Cryptography'
            ],
            'Cryptography Implementation Section': [
                'Testing Cryptographic Code',
                'Real-World Example: Secure Communication Module'
            ],
            'Migration and Integration Section': [
                'Migration Strategy: From C to Rust',
                'Debugging and Tooling'
            ]
        }
        
        mappings = []
        
        for section in self.sections:
            mapped = False
            for proposed_section, original_sections in proposed_structure.items():
                if section.title in original_sections:
                    mappings.append(ContentMapping(
                        original_section=section.title,
                        proposed_section=proposed_section,
                        action='keep',
                        reason='Direct mapping to proposed structure'
                    ))
                    mapped = True
                    break
            
            if not mapped:
                # Determine action based on content analysis
                action, reason = self._determine_action(section)
                mappings.append(ContentMapping(
                    original_section=section.title,
                    proposed_section='TBD',
                    action=action,
                    reason=reason
                ))
        
        self.mappings = mappings
        return mappings
    
    def _determine_action(self, section: Section) -> Tuple[str, str]:
        """Determine what action to take with a section."""
        content_lower = section.content.lower()
        
        # Check if section is too short or mostly empty
        if section.word_count < 50:
            return 'eliminate', 'Section too short, likely redundant'
        
        # Check for introduction/overview sections
        if 'introduction' in section.title.lower() or 'overview' in section.title.lower():
            return 'consolidate', 'Introduction content should be consolidated'
        
        # Check for table of contents
        if 'table of contents' in section.title.lower():
            return 'eliminate', 'TOC will be regenerated'
        
        # Default action
        return 'merge', 'Needs review for consolidation'
    
    def generate_report(self) -> Dict:
        """Generate comprehensive analysis report."""
        redundant_patterns = self.identify_redundant_patterns()
        
        report = {
            'document_stats': {
                'total_sections': len(self.sections),
                'total_words': sum(s.word_count for s in self.sections),
                'total_code_blocks': sum(len(s.code_blocks) for s in self.sections)
            },
            'section_analysis': [
                {
                    'title': s.title,
                    'level': s.level,
                    'word_count': s.word_count,
                    'code_blocks_count': len(s.code_blocks),
                    'line_range': f"{s.line_start}-{s.line_end}"
                }
                for s in self.sections
            ],
            'duplicate_content': [
                {
                    'sections': d.sections,
                    'similarity_score': d.similarity_score,
                    'content_preview': d.content[:100] + "..."
                }
                for d in self.duplicates
            ],
            'redundant_patterns': redundant_patterns,
            'content_mapping': [
                {
                    'original_section': m.original_section,
                    'proposed_section': m.proposed_section,
                    'action': m.action,
                    'reason': m.reason
                }
                for m in self.mappings
            ],
            'recommendations': self._generate_recommendations()
        }
        
        return report
    
    def _generate_recommendations(self) -> List[str]:
        """Generate specific recommendations for improvement."""
        recommendations = []
        
        # Check for sections that need consolidation
        consolidate_count = sum(1 for m in self.mappings if m.action == 'consolidate')
        if consolidate_count > 0:
            recommendations.append(f"Consolidate {consolidate_count} sections to reduce redundancy")
        
        # Check for duplicate patterns
        if len(self.duplicates) > 0:
            recommendations.append(f"Found {len(self.duplicates)} instances of duplicate content that should be merged")
        
        # Check for setup instructions scattered across sections
        setup_sections = [s.title for s in self.sections if 'setup' in s.title.lower() or 'install' in s.content.lower()]
        if len(setup_sections) > 1:
            recommendations.append(f"Consolidate setup instructions from {len(setup_sections)} sections into single Environment Setup section")
        
        # Check for memory management explanations
        memory_sections = [s.title for s in self.sections if any(keyword in s.content.lower() for keyword in ['ownership', 'borrowing', 'memory management'])]
        if len(memory_sections) > 2:
            recommendations.append(f"Consolidate memory management explanations from {len(memory_sections)} sections")
        
        return recommendations

def main():
    analyzer = ContentAnalyzer('embedded-rust-tutorial.md')
    
    print("Analyzing content structure...")
    
    # Analyze duplicates
    duplicates = analyzer.analyze_duplicates(similarity_threshold=0.6)
    print(f"Found {len(duplicates)} potential duplicate content blocks")
    
    # Generate content mapping
    mappings = analyzer.generate_content_mapping()
    print(f"Generated {len(mappings)} content mappings")
    
    # Generate comprehensive report
    report = analyzer.generate_report()
    
    # Save report to JSON file
    with open('content_analysis_report.json', 'w', encoding='utf-8') as f:
        json.dump(report, f, indent=2, ensure_ascii=False)
    
    # Print summary
    print("\n=== CONTENT ANALYSIS SUMMARY ===")
    print(f"Total sections: {report['document_stats']['total_sections']}")
    print(f"Total words: {report['document_stats']['total_words']}")
    print(f"Total code blocks: {report['document_stats']['total_code_blocks']}")
    
    print(f"\nDuplicate content instances: {len(report['duplicate_content'])}")
    print(f"Sections needing consolidation: {sum(1 for m in mappings if m.action == 'consolidate')}")
    print(f"Sections to eliminate: {sum(1 for m in mappings if m.action == 'eliminate')}")
    
    print("\n=== RECOMMENDATIONS ===")
    for rec in report['recommendations']:
        print(f"- {rec}")
    
    print(f"\nDetailed report saved to: content_analysis_report.json")

if __name__ == "__main__":
    main()