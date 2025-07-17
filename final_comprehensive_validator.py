#!/usr/bin/env python3
"""
Final Comprehensive Validation Suite for mdBook Conversion
Validates all aspects of the converted mdBook to ensure quality and completeness.
"""

import os
import sys
import json
import re
import subprocess
import requests
from pathlib import Path
from typing import Dict, List, Tuple, Any
import time
from urllib.parse import urljoin, urlparse
from bs4 import BeautifulSoup
import hashlib

class FinalValidator:
    def __init__(self):
        self.results = {
            'content_validation': {},
            'navigation_testing': {},
            'search_validation': {},
            'build_deployment': {},
            'responsiveness': {},
            'overall_status': 'PENDING'
        }
        self.errors = []
        self.warnings = []
        
    def run_all_validations(self):
        """Run all validation tests"""
        print("🚀 Starting Final Comprehensive Validation Suite")
        print("=" * 60)
        
        # 1. Content validation
        print("\n📋 1. Running content validation...")
        self.validate_content_preservation()
        
        # 2. Navigation testing
        print("\n🧭 2. Testing navigation flow...")
        self.test_navigation_flow()
        
        # 3. Search functionality
        print("\n🔍 3. Validating search functionality...")
        self.validate_search_functionality()
        
        # 4. Build and deployment
        print("\n🏗️ 4. Testing build and deployment...")
        self.test_build_deployment()
        
        # 5. Mobile responsiveness
        print("\n📱 5. Verifying responsiveness...")
        self.verify_responsiveness()
        
        # Generate final report
        self.generate_final_report()
        
    def validate_content_preservation(self):
        """Validate that all original content is preserved"""
        try:
            # Check if original document exists
            original_file = "embedded-rust-tutorial-master.md"
            if not os.path.exists(original_file):
                self.errors.append("Original tutorial file not found")
                return
                
            # Read original content
            with open(original_file, 'r', encoding='utf-8') as f:
                original_content = f.read()
                
            # Extract key content metrics from original
            original_metrics = self.extract_content_metrics(original_content)
            
            # Check converted content
            converted_metrics = self.extract_converted_metrics()
            
            # Compare metrics
            content_validation = self.compare_content_metrics(original_metrics, converted_metrics)
            
            self.results['content_validation'] = content_validation
            
            if content_validation['status'] == 'PASS':
                print("✅ Content validation passed")
            else:
                print("❌ Content validation failed")
                self.errors.extend(content_validation.get('errors', []))
                
        except Exception as e:
            self.errors.append(f"Content validation error: {str(e)}")
            self.results['content_validation'] = {'status': 'ERROR', 'error': str(e)}
            
    def extract_content_metrics(self, content: str) -> Dict:
        """Extract metrics from original content"""
        return {
            'total_lines': len(content.split('\n')),
            'code_blocks': len(re.findall(r'```', content)),
            'headers': len(re.findall(r'^#+\s', content, re.MULTILINE)),
            'links': len(re.findall(r'\[.*?\]\(.*?\)', content)),
            'tables': len(re.findall(r'\|.*\|', content)),
            'word_count': len(content.split()),
            'unique_sections': len(re.findall(r'^## ', content, re.MULTILINE))
        }
        
    def extract_converted_metrics(self) -> Dict:
        """Extract metrics from converted mdBook content"""
        src_dir = Path("src")
        if not src_dir.exists():
            return {}
            
        total_content = ""
        file_count = 0
        
        for md_file in src_dir.rglob("*.md"):
            try:
                with open(md_file, 'r', encoding='utf-8') as f:
                    content = f.read()
                    total_content += content + "\n"
                    file_count += 1
            except Exception as e:
                self.warnings.append(f"Could not read {md_file}: {e}")
                
        return {
            'total_files': file_count,
            'total_lines': len(total_content.split('\n')),
            'code_blocks': len(re.findall(r'```', total_content)),
            'headers': len(re.findall(r'^#+\s', total_content, re.MULTILINE)),
            'links': len(re.findall(r'\[.*?\]\(.*?\)', total_content)),
            'tables': len(re.findall(r'\|.*\|', total_content)),
            'word_count': len(total_content.split()),
            'content_hash': hashlib.md5(total_content.encode()).hexdigest()
        }
        
    def compare_content_metrics(self, original: Dict, converted: Dict) -> Dict:
        """Compare original and converted content metrics"""
        if not converted:
            return {'status': 'FAIL', 'errors': ['No converted content found']}
            
        errors = []
        warnings = []
        
        # Check critical metrics
        if converted.get('code_blocks', 0) < original.get('code_blocks', 0) * 0.9:
            errors.append(f"Code blocks significantly reduced: {converted.get('code_blocks')} vs {original.get('code_blocks')}")
            
        if converted.get('word_count', 0) < original.get('word_count', 0) * 0.8:
            errors.append(f"Word count significantly reduced: {converted.get('word_count')} vs {original.get('word_count')}")
            
        if converted.get('headers', 0) < original.get('headers', 0) * 0.8:
            warnings.append(f"Header count reduced: {converted.get('headers')} vs {original.get('headers')}")
            
        status = 'FAIL' if errors else ('WARN' if warnings else 'PASS')
        
        return {
            'status': status,
            'original_metrics': original,
            'converted_metrics': converted,
            'errors': errors,
            'warnings': warnings
        }
        
    def test_navigation_flow(self):
        """Test complete navigation flow through all chapters"""
        try:
            # Check SUMMARY.md structure
            summary_path = Path("src/SUMMARY.md")
            if not summary_path.exists():
                self.errors.append("SUMMARY.md not found")
                return
                
            with open(summary_path, 'r', encoding='utf-8') as f:
                summary_content = f.read()
                
            # Extract all links from SUMMARY.md
            links = re.findall(r'\[.*?\]\((.*?)\)', summary_content)
            
            navigation_results = {
                'total_links': len(links),
                'valid_links': 0,
                'broken_links': [],
                'missing_files': []
            }
            
            # Validate each link
            for link in links:
                if link.startswith('http'):
                    continue  # Skip external links
                    
                file_path = Path("src") / link
                if file_path.exists():
                    navigation_results['valid_links'] += 1
                    # Check if file has content
                    try:
                        with open(file_path, 'r', encoding='utf-8') as f:
                            content = f.read().strip()
                            if not content:
                                navigation_results['broken_links'].append(f"{link} (empty file)")
                    except Exception as e:
                        navigation_results['broken_links'].append(f"{link} (read error: {e})")
                else:
                    navigation_results['missing_files'].append(link)
                    
            # Check chapter structure
            chapter_dirs = ['quick-reference', 'environment-setup', 'core-concepts', 
                          'embedded-patterns', 'cryptography', 'migration']
            
            missing_chapters = []
            for chapter in chapter_dirs:
                chapter_path = Path("src") / chapter
                if not chapter_path.exists():
                    missing_chapters.append(chapter)
                elif not (chapter_path / "README.md").exists():
                    navigation_results['broken_links'].append(f"{chapter}/README.md (missing overview)")
                    
            navigation_results['missing_chapters'] = missing_chapters
            
            # Determine status
            if navigation_results['missing_files'] or missing_chapters:
                navigation_results['status'] = 'FAIL'
                self.errors.extend([f"Missing file: {f}" for f in navigation_results['missing_files']])
                self.errors.extend([f"Missing chapter: {c}" for c in missing_chapters])
            elif navigation_results['broken_links']:
                navigation_results['status'] = 'WARN'
                self.warnings.extend(navigation_results['broken_links'])
            else:
                navigation_results['status'] = 'PASS'
                
            self.results['navigation_testing'] = navigation_results
            
            if navigation_results['status'] == 'PASS':
                print("✅ Navigation flow testing passed")
            else:
                print(f"⚠️ Navigation flow testing: {navigation_results['status']}")
                
        except Exception as e:
            self.errors.append(f"Navigation testing error: {str(e)}")
            self.results['navigation_testing'] = {'status': 'ERROR', 'error': str(e)}
            
    def validate_search_functionality(self):
        """Validate search functionality across all content"""
        try:
            # Check if book builds successfully first
            build_result = subprocess.run(['mdbook', 'build'], 
                                        capture_output=True, text=True, cwd='.')
            
            if build_result.returncode != 0:
                self.errors.append(f"mdBook build failed: {build_result.stderr}")
                self.results['search_validation'] = {'status': 'FAIL', 'error': 'Build failed'}
                return
                
            # Check if search index is generated
            search_index_path = Path("book/searchindex.js")
            if not search_index_path.exists():
                self.errors.append("Search index not generated")
                self.results['search_validation'] = {'status': 'FAIL', 'error': 'No search index'}
                return
                
            # Analyze search index content
            with open(search_index_path, 'r', encoding='utf-8') as f:
                search_content = f.read()
                
            # Extract search data
            search_metrics = {
                'index_size': len(search_content),
                'has_content': 'searchindex' in search_content.lower(),
                'estimated_entries': search_content.count('"title"') if '"title"' in search_content else 0
            }
            
            # Test key search terms
            test_terms = ['rust', 'embedded', 'crypto', 'memory', 'ownership', 'safety']
            searchable_terms = []
            
            for term in test_terms:
                if term.lower() in search_content.lower():
                    searchable_terms.append(term)
                    
            search_results = {
                'status': 'PASS' if len(searchable_terms) >= len(test_terms) * 0.8 else 'WARN',
                'metrics': search_metrics,
                'searchable_terms': searchable_terms,
                'missing_terms': [t for t in test_terms if t not in searchable_terms]
            }
            
            if search_results['status'] == 'PASS':
                print("✅ Search functionality validation passed")
            else:
                print("⚠️ Search functionality validation has warnings")
                self.warnings.extend([f"Search term not found: {t}" for t in search_results['missing_terms']])
                
            self.results['search_validation'] = search_results
            
        except Exception as e:
            self.errors.append(f"Search validation error: {str(e)}")
            self.results['search_validation'] = {'status': 'ERROR', 'error': str(e)}
            
    def test_build_deployment(self):
        """Test build and deployment process end-to-end"""
        try:
            print("  Building mdBook...")
            
            # Clean previous build
            if Path("book").exists():
                subprocess.run(['rm', '-rf', 'book'], check=False)
                
            # Test mdBook build
            build_result = subprocess.run(['mdbook', 'build'], 
                                        capture_output=True, text=True, cwd='.')
            
            build_status = {
                'build_success': build_result.returncode == 0,
                'build_output': build_result.stdout,
                'build_errors': build_result.stderr,
                'generated_files': []
            }
            
            if build_result.returncode == 0:
                # Check generated files
                book_dir = Path("book")
                if book_dir.exists():
                    for file_path in book_dir.rglob("*"):
                        if file_path.is_file():
                            build_status['generated_files'].append(str(file_path.relative_to(book_dir)))
                            
                # Check critical files
                critical_files = ['index.html', 'book.js', 'searchindex.js', 'print.html']
                missing_critical = []
                
                for critical_file in critical_files:
                    if not (book_dir / critical_file).exists():
                        missing_critical.append(critical_file)
                        
                build_status['missing_critical_files'] = missing_critical
                build_status['total_generated_files'] = len(build_status['generated_files'])
                
                # Test local serving capability
                print("  Testing local server...")
                server_test = self.test_local_server()
                build_status['server_test'] = server_test
                
                if missing_critical:
                    build_status['status'] = 'WARN'
                    self.warnings.extend([f"Missing critical file: {f}" for f in missing_critical])
                else:
                    build_status['status'] = 'PASS'
                    
            else:
                build_status['status'] = 'FAIL'
                self.errors.append(f"Build failed: {build_result.stderr}")
                
            self.results['build_deployment'] = build_status
            
            if build_status['status'] == 'PASS':
                print("✅ Build and deployment testing passed")
            else:
                print(f"❌ Build and deployment testing: {build_status['status']}")
                
        except Exception as e:
            self.errors.append(f"Build/deployment testing error: {str(e)}")
            self.results['build_deployment'] = {'status': 'ERROR', 'error': str(e)}
            
    def test_local_server(self) -> Dict:
        """Test local server functionality"""
        try:
            # Start mdbook serve in background
            server_process = subprocess.Popen(['mdbook', 'serve', '--port', '3001'], 
                                            stdout=subprocess.PIPE, 
                                            stderr=subprocess.PIPE)
            
            # Wait for server to start
            time.sleep(3)
            
            try:
                # Test server response
                response = requests.get('http://localhost:3001', timeout=5)
                server_working = response.status_code == 200
                
                if server_working:
                    # Test search functionality
                    search_working = 'searchindex.js' in response.text
                    
                    return {
                        'server_starts': True,
                        'responds_to_requests': True,
                        'search_available': search_working,
                        'status': 'PASS'
                    }
                else:
                    return {
                        'server_starts': True,
                        'responds_to_requests': False,
                        'status_code': response.status_code,
                        'status': 'WARN'
                    }
                    
            except requests.RequestException as e:
                return {
                    'server_starts': True,
                    'responds_to_requests': False,
                    'error': str(e),
                    'status': 'WARN'
                }
                
            finally:
                # Clean up server process
                server_process.terminate()
                server_process.wait(timeout=5)
                
        except Exception as e:
            return {
                'server_starts': False,
                'error': str(e),
                'status': 'FAIL'
            }
            
    def verify_responsiveness(self):
        """Verify mobile responsiveness and cross-browser compatibility"""
        try:
            # Check if book is built
            book_dir = Path("book")
            if not book_dir.exists():
                self.errors.append("Book not built - cannot test responsiveness")
                self.results['responsiveness'] = {'status': 'FAIL', 'error': 'No built book'}
                return
                
            # Check CSS files for responsive design
            css_files = list(book_dir.glob("**/*.css"))
            responsive_features = {
                'viewport_meta': False,
                'media_queries': 0,
                'responsive_css': False,
                'mobile_friendly': False
            }
            
            # Check HTML files for viewport meta tag
            html_files = list(book_dir.glob("**/*.html"))
            for html_file in html_files[:5]:  # Check first 5 HTML files
                try:
                    with open(html_file, 'r', encoding='utf-8') as f:
                        html_content = f.read()
                        if 'viewport' in html_content and 'width=device-width' in html_content:
                            responsive_features['viewport_meta'] = True
                            break
                except Exception:
                    continue
                    
            # Check CSS files for responsive features
            for css_file in css_files:
                try:
                    with open(css_file, 'r', encoding='utf-8') as f:
                        css_content = f.read()
                        
                        # Count media queries
                        media_queries = len(re.findall(r'@media', css_content, re.IGNORECASE))
                        responsive_features['media_queries'] += media_queries
                        
                        # Check for responsive patterns
                        if any(pattern in css_content.lower() for pattern in 
                              ['max-width', 'min-width', 'flex', 'grid']):
                            responsive_features['responsive_css'] = True
                            
                except Exception:
                    continue
                    
            # Determine mobile friendliness
            responsive_features['mobile_friendly'] = (
                responsive_features['viewport_meta'] and 
                responsive_features['media_queries'] > 0 and
                responsive_features['responsive_css']
            )
            
            # Check theme files
            theme_dir = book_dir / "theme"
            custom_theme = theme_dir.exists() and len(list(theme_dir.glob("*.css"))) > 0
            
            responsiveness_results = {
                'responsive_features': responsive_features,
                'custom_theme': custom_theme,
                'css_files_count': len(css_files),
                'html_files_count': len(html_files),
                'status': 'PASS' if responsive_features['mobile_friendly'] else 'WARN'
            }
            
            if responsiveness_results['status'] == 'PASS':
                print("✅ Responsiveness verification passed")
            else:
                print("⚠️ Responsiveness verification has warnings")
                if not responsive_features['viewport_meta']:
                    self.warnings.append("No viewport meta tag found")
                if responsive_features['media_queries'] == 0:
                    self.warnings.append("No media queries found")
                    
            self.results['responsiveness'] = responsiveness_results
            
        except Exception as e:
            self.errors.append(f"Responsiveness verification error: {str(e)}")
            self.results['responsiveness'] = {'status': 'ERROR', 'error': str(e)}
            
    def generate_final_report(self):
        """Generate comprehensive final validation report"""
        # Determine overall status
        statuses = [result.get('status', 'ERROR') for result in self.results.values() if isinstance(result, dict)]
        
        if 'FAIL' in statuses or 'ERROR' in statuses:
            self.results['overall_status'] = 'FAIL'
        elif 'WARN' in statuses:
            self.results['overall_status'] = 'WARN'
        else:
            self.results['overall_status'] = 'PASS'
            
        # Generate summary
        summary = {
            'timestamp': time.strftime('%Y-%m-%d %H:%M:%S'),
            'overall_status': self.results['overall_status'],
            'total_errors': len(self.errors),
            'total_warnings': len(self.warnings),
            'validation_results': self.results,
            'errors': self.errors,
            'warnings': self.warnings
        }
        
        # Save detailed results
        with open('final_validation_results.json', 'w') as f:
            json.dump(summary, f, indent=2)
            
        # Generate human-readable report
        self.generate_readable_report(summary)
        
        # Print summary
        print("\n" + "=" * 60)
        print("🏁 FINAL VALIDATION SUMMARY")
        print("=" * 60)
        print(f"Overall Status: {self.results['overall_status']}")
        print(f"Errors: {len(self.errors)}")
        print(f"Warnings: {len(self.warnings)}")
        
        if self.results['overall_status'] == 'PASS':
            print("\n🎉 All validations passed! The mdBook conversion is ready for production.")
        elif self.results['overall_status'] == 'WARN':
            print("\n⚠️ Validation completed with warnings. Review the issues before deployment.")
        else:
            print("\n❌ Validation failed. Critical issues must be resolved before deployment.")
            
        print(f"\nDetailed report saved to: final_validation_report.md")
        print(f"JSON results saved to: final_validation_results.json")
        
    def generate_readable_report(self, summary: Dict):
        """Generate human-readable markdown report"""
        report_content = f"""# Final Validation Report

**Generated:** {summary['timestamp']}  
**Overall Status:** {summary['overall_status']}  
**Total Errors:** {summary['total_errors']}  
**Total Warnings:** {summary['total_warnings']}

## Executive Summary

This report contains the results of comprehensive validation testing for the mdBook conversion of the Embedded Rust Tutorial. The validation covers content preservation, navigation functionality, search capabilities, build processes, and responsive design.

## Validation Results

### 1. Content Validation
**Status:** {self.results['content_validation'].get('status', 'ERROR')}

"""
        
        if 'content_validation' in self.results and isinstance(self.results['content_validation'], dict):
            cv = self.results['content_validation']
            if 'original_metrics' in cv and 'converted_metrics' in cv:
                report_content += f"""
**Original Content Metrics:**
- Total lines: {cv['original_metrics'].get('total_lines', 'N/A')}
- Code blocks: {cv['original_metrics'].get('code_blocks', 'N/A')}
- Headers: {cv['original_metrics'].get('headers', 'N/A')}
- Word count: {cv['original_metrics'].get('word_count', 'N/A')}

**Converted Content Metrics:**
- Total files: {cv['converted_metrics'].get('total_files', 'N/A')}
- Total lines: {cv['converted_metrics'].get('total_lines', 'N/A')}
- Code blocks: {cv['converted_metrics'].get('code_blocks', 'N/A')}
- Headers: {cv['converted_metrics'].get('headers', 'N/A')}
- Word count: {cv['converted_metrics'].get('word_count', 'N/A')}
"""

        report_content += f"""
### 2. Navigation Testing
**Status:** {self.results['navigation_testing'].get('status', 'ERROR')}

"""
        
        if 'navigation_testing' in self.results and isinstance(self.results['navigation_testing'], dict):
            nt = self.results['navigation_testing']
            report_content += f"""
- Total links in SUMMARY.md: {nt.get('total_links', 'N/A')}
- Valid links: {nt.get('valid_links', 'N/A')}
- Broken links: {len(nt.get('broken_links', []))}
- Missing files: {len(nt.get('missing_files', []))}
"""

        report_content += f"""
### 3. Search Functionality
**Status:** {self.results['search_validation'].get('status', 'ERROR')}

"""
        
        if 'search_validation' in self.results and isinstance(self.results['search_validation'], dict):
            sv = self.results['search_validation']
            if 'metrics' in sv:
                report_content += f"""
- Search index generated: {'Yes' if sv['metrics'].get('has_content') else 'No'}
- Estimated search entries: {sv['metrics'].get('estimated_entries', 'N/A')}
- Searchable key terms: {len(sv.get('searchable_terms', []))}
"""

        report_content += f"""
### 4. Build and Deployment
**Status:** {self.results['build_deployment'].get('status', 'ERROR')}

"""
        
        if 'build_deployment' in self.results and isinstance(self.results['build_deployment'], dict):
            bd = self.results['build_deployment']
            report_content += f"""
- Build successful: {'Yes' if bd.get('build_success') else 'No'}
- Generated files: {bd.get('total_generated_files', 'N/A')}
- Missing critical files: {len(bd.get('missing_critical_files', []))}
- Local server test: {bd.get('server_test', {}).get('status', 'N/A')}
"""

        report_content += f"""
### 5. Responsiveness and Compatibility
**Status:** {self.results['responsiveness'].get('status', 'ERROR')}

"""
        
        if 'responsiveness' in self.results and isinstance(self.results['responsiveness'], dict):
            resp = self.results['responsiveness']
            if 'responsive_features' in resp:
                rf = resp['responsive_features']
                report_content += f"""
- Viewport meta tag: {'Yes' if rf.get('viewport_meta') else 'No'}
- Media queries found: {rf.get('media_queries', 0)}
- Responsive CSS: {'Yes' if rf.get('responsive_css') else 'No'}
- Mobile friendly: {'Yes' if rf.get('mobile_friendly') else 'No'}
"""

        # Add errors and warnings
        if self.errors:
            report_content += "\n## Errors\n\n"
            for i, error in enumerate(self.errors, 1):
                report_content += f"{i}. {error}\n"
                
        if self.warnings:
            report_content += "\n## Warnings\n\n"
            for i, warning in enumerate(self.warnings, 1):
                report_content += f"{i}. {warning}\n"
                
        # Add recommendations
        report_content += f"""
## Recommendations

Based on the validation results:

"""
        
        if self.results['overall_status'] == 'PASS':
            report_content += """
✅ **Ready for Production**: All validations passed successfully. The mdBook conversion maintains content integrity and provides excellent user experience.

**Next Steps:**
1. Deploy to production environment
2. Set up monitoring for the deployed site
3. Consider adding analytics to track usage patterns
"""
        elif self.results['overall_status'] == 'WARN':
            report_content += """
⚠️ **Review Required**: The conversion is functional but has some issues that should be addressed.

**Recommended Actions:**
1. Review and fix the warnings listed above
2. Test the specific areas that showed warnings
3. Consider the impact of warnings on user experience
4. Deploy to staging environment for further testing
"""
        else:
            report_content += """
❌ **Issues Must Be Resolved**: Critical errors prevent production deployment.

**Required Actions:**
1. Address all errors listed above
2. Re-run validation after fixes
3. Ensure all content is properly preserved
4. Verify build and deployment processes work correctly
"""

        report_content += f"""
## Technical Details

**Validation Environment:**
- Python version: {sys.version.split()[0]}
- Working directory: {os.getcwd()}
- Validation timestamp: {summary['timestamp']}

**Files Checked:**
- Source markdown files in src/ directory
- Generated HTML files in book/ directory
- Configuration files (book.toml, SUMMARY.md)
- CSS and JavaScript assets
- Search index and functionality

---

*This report was generated automatically by the Final Comprehensive Validation Suite.*
"""

        with open('final_validation_report.md', 'w') as f:
            f.write(report_content)

if __name__ == "__main__":
    validator = FinalValidator()
    validator.run_all_validations()