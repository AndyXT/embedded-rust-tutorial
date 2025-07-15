#!/usr/bin/env python3
"""
Redundancy Detection System
Advanced duplicate content detection for technical documentation
"""

import re
import json
import hashlib
from pathlib import Path
from typing import Dict, List, Set, Tuple, Optional
from dataclasses import dataclass, asdict
from collections import defaultdict, Counter
import difflib

@dataclass
class ContentSegment:
    """Represents a segment of content for analysis"""
    content: str
    normalized_content: str
    hash: str
    line_start: int
    line_end: int
    section: str
    segment_type: str  # 'paragraph', 'code', 'list', 'table'
    word_count: int
    key_terms: Set[str]

@dataclass
class DuplicateMatch:
    """Represents a duplicate content match"""
    segment1: ContentSegment
    segment2: ContentSegment
    similarity_score: float
    match_type: str  # 'exact', 'near_exact', 'similar', 'conceptual'
    common_phrases: List[str]
    diff_summary: str

@dataclass
class RedundancyReport:
    """Complete redundancy analysis report"""
    total_segments: int
    exact_duplicates: List[DuplicateMatch]
    near_duplicates: List[DuplicateMatch]
    similar_content: List[DuplicateMatch]
    conceptual_overlaps: List[DuplicateMatch]
    redundancy_statistics: Dict[str, int]
    section_analysis: Dict[str, Dict[str, int]]

class RedundancyDetector:
    """Detects redundant and duplicate content in technical documentation"""
    
    def __init__(self, document_path: str):
        self.document_path = Path(document_path)
        self.content = ""
        self.lines = []
        self.segments: List[ContentSegment] = []
        
        # Technical terms that indicate crypto/embedded content
        self.technical_terms = {
            'crypto', 'encryption', 'decryption', 'key', 'cipher', 'hash', 'aes', 'rsa',
            'embedded', 'microcontroller', 'cortex', 'arm', 'interrupt', 'dma', 'gpio',
            'rust', 'ownership', 'borrowing', 'lifetime', 'unsafe', 'memory', 'pointer',
            'buffer', 'array', 'slice', 'vector', 'string', 'struct', 'enum', 'trait'
        }
    
    def load_document(self) -> bool:
        """Load the document content"""
        try:
            with open(self.document_path, 'r', encoding='utf-8') as f:
                self.content = f.read()
                self.lines = self.content.split('\n')
            return True
        except Exception as e:
            print(f"Error loading document: {e}")
            return False
    
    def extract_content_segments(self):
        """Extract and classify content segments"""
        current_section = "Introduction"
        i = 0
        
        while i < len(self.lines):
            line = self.lines[i]
            
            # Track current section
            if line.startswith('#'):
                header_match = re.match(r'^#+\s+(.+?)(?:\s*\{#[^}]+\})?$', line)
                if header_match:
                    current_section = header_match.group(1).strip()
                i += 1
                continue
            
            # Skip empty lines
            if not line.strip():
                i += 1
                continue
            
            # Detect segment type and extract
            if line.startswith('```'):
                # Code block
                segment, end_line = self.extract_code_block(i, current_section)
                if segment:
                    self.segments.append(segment)
                i = end_line + 1
            elif line.startswith('|') and '|' in line[1:]:
                # Table
                segment, end_line = self.extract_table(i, current_section)
                if segment:
                    self.segments.append(segment)
                i = end_line + 1
            elif line.startswith(('- ', '* ', '+ ')) or re.match(r'^\d+\.', line):
                # List
                segment, end_line = self.extract_list(i, current_section)
                if segment:
                    self.segments.append(segment)
                i = end_line + 1
            else:
                # Paragraph
                segment, end_line = self.extract_paragraph(i, current_section)
                if segment:
                    self.segments.append(segment)
                i = end_line + 1
    
    def extract_code_block(self, start_line: int, section: str) -> Tuple[Optional[ContentSegment], int]:
        """Extract a code block segment"""
        if not self.lines[start_line].startswith('```'):
            return None, start_line
        
        language = self.lines[start_line][3:].strip()
        content_lines = []
        i = start_line + 1
        
        while i < len(self.lines) and not self.lines[i].startswith('```'):
            content_lines.append(self.lines[i])
            i += 1
        
        if i >= len(self.lines):
            return None, start_line
        
        content = '\n'.join(content_lines)
        if not content.strip():
            return None, i
        
        normalized = self.normalize_code_content(content)
        
        return ContentSegment(
            content=content,
            normalized_content=normalized,
            hash=hashlib.md5(normalized.encode()).hexdigest(),
            line_start=start_line + 1,
            line_end=i + 1,
            section=section,
            segment_type=f'code_{language}' if language else 'code',
            word_count=len(content.split()),
            key_terms=self.extract_key_terms(content)
        ), i
    
    def extract_table(self, start_line: int, section: str) -> Tuple[Optional[ContentSegment], int]:
        """Extract a table segment"""
        content_lines = []
        i = start_line
        
        while i < len(self.lines) and (self.lines[i].startswith('|') or self.lines[i].strip().startswith('|')):
            content_lines.append(self.lines[i])
            i += 1
        
        if len(content_lines) < 2:  # Need at least header and separator
            return None, start_line
        
        content = '\n'.join(content_lines)
        normalized = self.normalize_table_content(content)
        
        return ContentSegment(
            content=content,
            normalized_content=normalized,
            hash=hashlib.md5(normalized.encode()).hexdigest(),
            line_start=start_line + 1,
            line_end=i,
            section=section,
            segment_type='table',
            word_count=len(content.split()),
            key_terms=self.extract_key_terms(content)
        ), i - 1
    
    def extract_list(self, start_line: int, section: str) -> Tuple[Optional[ContentSegment], int]:
        """Extract a list segment"""
        content_lines = []
        i = start_line
        
        while i < len(self.lines):
            line = self.lines[i]
            if not line.strip():
                i += 1
                continue
            
            # Check if it's a list item or continuation
            if (line.startswith(('- ', '* ', '+ ')) or 
                re.match(r'^\d+\.', line) or 
                line.startswith('  ')):  # Indented continuation
                content_lines.append(line)
                i += 1
            else:
                break
        
        if not content_lines:
            return None, start_line
        
        content = '\n'.join(content_lines)
        normalized = self.normalize_text_content(content)
        
        return ContentSegment(
            content=content,
            normalized_content=normalized,
            hash=hashlib.md5(normalized.encode()).hexdigest(),
            line_start=start_line + 1,
            line_end=i,
            section=section,
            segment_type='list',
            word_count=len(content.split()),
            key_terms=self.extract_key_terms(content)
        ), i - 1
    
    def extract_paragraph(self, start_line: int, section: str) -> Tuple[Optional[ContentSegment], int]:
        """Extract a paragraph segment"""
        content_lines = []
        i = start_line
        
        while i < len(self.lines):
            line = self.lines[i]
            
            # Stop at empty line, header, or special formatting
            if (not line.strip() or 
                line.startswith('#') or 
                line.startswith('```') or 
                line.startswith('|') or
                line.startswith(('- ', '* ', '+ ')) or
                re.match(r'^\d+\.', line)):
                break
            
            content_lines.append(line)
            i += 1
        
        if not content_lines:
            return None, start_line
        
        content = '\n'.join(content_lines)
        
        # Skip very short paragraphs
        if len(content.split()) < 5:
            return None, i - 1
        
        normalized = self.normalize_text_content(content)
        
        return ContentSegment(
            content=content,
            normalized_content=normalized,
            hash=hashlib.md5(normalized.encode()).hexdigest(),
            line_start=start_line + 1,
            line_end=i,
            section=section,
            segment_type='paragraph',
            word_count=len(content.split()),
            key_terms=self.extract_key_terms(content)
        ), i - 1
    
    def normalize_code_content(self, content: str) -> str:
        """Normalize code content for comparison"""
        # Remove comments
        lines = []
        for line in content.split('\n'):
            # Remove Rust comments
            line = re.sub(r'//.*$', '', line)
            line = re.sub(r'/\*.*?\*/', '', line)
            # Remove extra whitespace but preserve structure
            line = re.sub(r'\s+', ' ', line.strip())
            if line:
                lines.append(line)
        
        return '\n'.join(lines)
    
    def normalize_table_content(self, content: str) -> str:
        """Normalize table content for comparison"""
        lines = []
        for line in content.split('\n'):
            # Extract cell content, ignoring formatting
            cells = [cell.strip() for cell in line.split('|')[1:-1]]
            if cells and not all(cell in ['', '---', ':---', '---:', ':---:'] for cell in cells):
                lines.append('|'.join(cells))
        
        return '\n'.join(lines)
    
    def normalize_text_content(self, content: str) -> str:
        """Normalize text content for comparison"""
        # Remove markdown formatting
        text = re.sub(r'\*\*(.*?)\*\*', r'\1', content)  # Bold
        text = re.sub(r'\*(.*?)\*', r'\1', text)  # Italic
        text = re.sub(r'`(.*?)`', r'\1', text)  # Inline code
        text = re.sub(r'\[([^\]]+)\]\([^)]+\)', r'\1', text)  # Links
        
        # Normalize whitespace
        text = re.sub(r'\s+', ' ', text)
        
        # Convert to lowercase for comparison
        return text.lower().strip()
    
    def extract_key_terms(self, content: str) -> Set[str]:
        """Extract key technical terms from content"""
        words = re.findall(r'\b\w+\b', content.lower())
        key_terms = set()
        
        for word in words:
            if word in self.technical_terms or len(word) > 6:
                key_terms.add(word)
        
        return key_terms
    
    def find_exact_duplicates(self) -> List[DuplicateMatch]:
        """Find segments with identical content"""
        duplicates = []
        hash_groups = defaultdict(list)
        
        # Group by hash
        for segment in self.segments:
            hash_groups[segment.hash].append(segment)
        
        # Find groups with multiple segments
        for hash_val, segments in hash_groups.items():
            if len(segments) > 1:
                for i in range(len(segments)):
                    for j in range(i + 1, len(segments)):
                        duplicates.append(DuplicateMatch(
                            segment1=segments[i],
                            segment2=segments[j],
                            similarity_score=1.0,
                            match_type='exact',
                            common_phrases=[],
                            diff_summary="Identical content"
                        ))
        
        return duplicates
    
    def find_near_duplicates(self) -> List[DuplicateMatch]:
        """Find segments with very similar content (90%+ similarity)"""
        near_duplicates = []
        
        for i in range(len(self.segments)):
            for j in range(i + 1, len(self.segments)):
                seg1, seg2 = self.segments[i], self.segments[j]
                
                # Skip if already exact duplicates
                if seg1.hash == seg2.hash:
                    continue
                
                # Only compare segments of same type
                if seg1.segment_type != seg2.segment_type:
                    continue
                
                similarity = self.calculate_text_similarity(
                    seg1.normalized_content, 
                    seg2.normalized_content
                )
                
                if similarity >= 0.9:
                    common_phrases = self.find_common_phrases(seg1.content, seg2.content)
                    diff_summary = self.generate_diff_summary(seg1.content, seg2.content)
                    
                    near_duplicates.append(DuplicateMatch(
                        segment1=seg1,
                        segment2=seg2,
                        similarity_score=similarity,
                        match_type='near_exact',
                        common_phrases=common_phrases,
                        diff_summary=diff_summary
                    ))
        
        return near_duplicates
    
    def find_similar_content(self) -> List[DuplicateMatch]:
        """Find segments with similar content (70-90% similarity)"""
        similar_content = []
        
        for i in range(len(self.segments)):
            for j in range(i + 1, len(self.segments)):
                seg1, seg2 = self.segments[i], self.segments[j]
                
                # Skip if different types or already processed
                if seg1.segment_type != seg2.segment_type or seg1.hash == seg2.hash:
                    continue
                
                similarity = self.calculate_text_similarity(
                    seg1.normalized_content, 
                    seg2.normalized_content
                )
                
                if 0.7 <= similarity < 0.9:
                    common_phrases = self.find_common_phrases(seg1.content, seg2.content)
                    diff_summary = self.generate_diff_summary(seg1.content, seg2.content)
                    
                    similar_content.append(DuplicateMatch(
                        segment1=seg1,
                        segment2=seg2,
                        similarity_score=similarity,
                        match_type='similar',
                        common_phrases=common_phrases,
                        diff_summary=diff_summary
                    ))
        
        return similar_content
    
    def find_conceptual_overlaps(self) -> List[DuplicateMatch]:
        """Find segments that cover similar concepts"""
        conceptual_overlaps = []
        
        for i in range(len(self.segments)):
            for j in range(i + 1, len(self.segments)):
                seg1, seg2 = self.segments[i], self.segments[j]
                
                # Skip if same section or already processed
                if seg1.section == seg2.section or seg1.hash == seg2.hash:
                    continue
                
                # Check for conceptual overlap based on key terms
                common_terms = seg1.key_terms.intersection(seg2.key_terms)
                if len(common_terms) >= 3:  # At least 3 common technical terms
                    term_overlap = len(common_terms) / len(seg1.key_terms.union(seg2.key_terms))
                    
                    if term_overlap >= 0.4:  # 40% term overlap
                        conceptual_overlaps.append(DuplicateMatch(
                            segment1=seg1,
                            segment2=seg2,
                            similarity_score=term_overlap,
                            match_type='conceptual',
                            common_phrases=list(common_terms),
                            diff_summary=f"Shares {len(common_terms)} key terms: {', '.join(list(common_terms)[:5])}"
                        ))
        
        return conceptual_overlaps
    
    def calculate_text_similarity(self, text1: str, text2: str) -> float:
        """Calculate similarity between two text segments"""
        # Use difflib for sequence matching
        matcher = difflib.SequenceMatcher(None, text1, text2)
        return matcher.ratio()
    
    def find_common_phrases(self, text1: str, text2: str, min_length: int = 10) -> List[str]:
        """Find common phrases between two texts"""
        common_phrases = []
        
        # Split into sentences
        sentences1 = re.split(r'[.!?]+', text1)
        sentences2 = re.split(r'[.!?]+', text2)
        
        for sent1 in sentences1:
            sent1 = sent1.strip()
            if len(sent1) < min_length:
                continue
                
            for sent2 in sentences2:
                sent2 = sent2.strip()
                if len(sent2) < min_length:
                    continue
                
                # Check for high similarity
                similarity = difflib.SequenceMatcher(None, sent1.lower(), sent2.lower()).ratio()
                if similarity > 0.8:
                    common_phrases.append(sent1[:100] + "..." if len(sent1) > 100 else sent1)
        
        return common_phrases[:5]  # Limit to 5 phrases
    
    def generate_diff_summary(self, text1: str, text2: str) -> str:
        """Generate a summary of differences between two texts"""
        lines1 = text1.split('\n')
        lines2 = text2.split('\n')
        
        differ = difflib.unified_diff(lines1, lines2, lineterm='', n=1)
        diff_lines = list(differ)
        
        if len(diff_lines) <= 10:
            return '\n'.join(diff_lines)
        else:
            return f"Large diff with {len(diff_lines)} changes. First few:\n" + '\n'.join(diff_lines[:5])
    
    def analyze_redundancy_statistics(self, exact: List[DuplicateMatch], 
                                    near: List[DuplicateMatch], 
                                    similar: List[DuplicateMatch], 
                                    conceptual: List[DuplicateMatch]) -> Dict[str, int]:
        """Analyze redundancy statistics"""
        stats = {
            'total_segments': len(self.segments),
            'exact_duplicates': len(exact),
            'near_duplicates': len(near),
            'similar_content': len(similar),
            'conceptual_overlaps': len(conceptual),
            'total_redundancy_issues': len(exact) + len(near) + len(similar) + len(conceptual),
        }
        
        # Calculate by segment type
        segment_types = Counter(seg.segment_type for seg in self.segments)
        for seg_type, count in segment_types.items():
            stats[f'{seg_type}_segments'] = count
        
        return stats
    
    def analyze_sections(self) -> Dict[str, Dict[str, int]]:
        """Analyze redundancy by section"""
        section_analysis = defaultdict(lambda: defaultdict(int))
        
        for segment in self.segments:
            section_analysis[segment.section]['total_segments'] += 1
            section_analysis[segment.section][segment.segment_type] += 1
        
        return dict(section_analysis)
    
    def run_analysis(self) -> RedundancyReport:
        """Run complete redundancy analysis"""
        print("Loading document...")
        if not self.load_document():
            return RedundancyReport(0, [], [], [], [], {}, {})
        
        print("Extracting content segments...")
        self.extract_content_segments()
        print(f"Found {len(self.segments)} content segments")
        
        print("Finding exact duplicates...")
        exact_duplicates = self.find_exact_duplicates()
        print(f"Found {len(exact_duplicates)} exact duplicates")
        
        print("Finding near duplicates...")
        near_duplicates = self.find_near_duplicates()
        print(f"Found {len(near_duplicates)} near duplicates")
        
        print("Finding similar content...")
        similar_content = self.find_similar_content()
        print(f"Found {len(similar_content)} similar content pairs")
        
        print("Finding conceptual overlaps...")
        conceptual_overlaps = self.find_conceptual_overlaps()
        print(f"Found {len(conceptual_overlaps)} conceptual overlaps")
        
        print("Analyzing statistics...")
        redundancy_statistics = self.analyze_redundancy_statistics(
            exact_duplicates, near_duplicates, similar_content, conceptual_overlaps
        )
        section_analysis = self.analyze_sections()
        
        return RedundancyReport(
            total_segments=len(self.segments),
            exact_duplicates=exact_duplicates,
            near_duplicates=near_duplicates,
            similar_content=similar_content,
            conceptual_overlaps=conceptual_overlaps,
            redundancy_statistics=redundancy_statistics,
            section_analysis=section_analysis
        )
    
    def generate_report(self, report: RedundancyReport, output_file: str = "redundancy_report.json"):
        """Generate detailed redundancy report"""
        report_data = {
            "summary": {
                "total_segments": report.total_segments,
                "exact_duplicates": len(report.exact_duplicates),
                "near_duplicates": len(report.near_duplicates),
                "similar_content": len(report.similar_content),
                "conceptual_overlaps": len(report.conceptual_overlaps),
                "redundancy_percentage": (len(report.exact_duplicates) + len(report.near_duplicates)) / report.total_segments * 100 if report.total_segments > 0 else 0
            },
            "exact_duplicates": [asdict(match) for match in report.exact_duplicates],
            "near_duplicates": [asdict(match) for match in report.near_duplicates],
            "similar_content": [asdict(match) for match in report.similar_content],
            "conceptual_overlaps": [asdict(match) for match in report.conceptual_overlaps],
            "redundancy_statistics": report.redundancy_statistics,
            "section_analysis": report.section_analysis
        }
        
        with open(output_file, 'w') as f:
            json.dump(report_data, f, indent=2, default=str)
        
        print(f"Redundancy report saved to {output_file}")
        return report_data

def main():
    """Main entry point"""
    import sys
    
    if len(sys.argv) != 2:
        print("Usage: python redundancy_detector.py <document_path>")
        sys.exit(1)
    
    document_path = sys.argv[1]
    
    if not Path(document_path).exists():
        print(f"Error: Document {document_path} not found")
        sys.exit(1)
    
    detector = RedundancyDetector(document_path)
    report = detector.run_analysis()
    
    # Generate detailed report
    detector.generate_report(report)
    
    # Print summary
    print("\n" + "="*60)
    print("REDUNDANCY ANALYSIS SUMMARY")
    print("="*60)
    print(f"Total Content Segments: {report.total_segments}")
    print(f"Exact Duplicates: {len(report.exact_duplicates)}")
    print(f"Near Duplicates: {len(report.near_duplicates)}")
    print(f"Similar Content: {len(report.similar_content)}")
    print(f"Conceptual Overlaps: {len(report.conceptual_overlaps)}")
    
    redundancy_rate = (len(report.exact_duplicates) + len(report.near_duplicates)) / report.total_segments * 100 if report.total_segments > 0 else 0
    print(f"Redundancy Rate: {redundancy_rate:.1f}%")
    
    if report.exact_duplicates:
        print("\nEXACT DUPLICATES:")
        for match in report.exact_duplicates[:5]:
            print(f"  🔄 {match.segment1.section} (line {match.segment1.line_start}) ↔ {match.segment2.section} (line {match.segment2.line_start})")
    
    if report.near_duplicates:
        print("\nNEAR DUPLICATES:")
        for match in report.near_duplicates[:5]:
            print(f"  ⚠️  {match.segment1.section} (line {match.segment1.line_start}) ↔ {match.segment2.section} (line {match.segment2.line_start}) - {match.similarity_score:.1%} similar")
    
    if redundancy_rate < 5:
        print("\n✅ Low redundancy detected - document is well-organized!")
    elif redundancy_rate < 15:
        print("\n⚠️  Moderate redundancy detected - consider consolidation")
    else:
        print("\n❌ High redundancy detected - significant consolidation needed")

if __name__ == "__main__":
    main()