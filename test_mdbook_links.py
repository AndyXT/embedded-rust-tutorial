#!/usr/bin/env python3
"""
Test script to verify mdBook links work correctly.

This script performs comprehensive testing of the mdBook structure
to ensure all links and cross-references work as expected.
"""

import os
import subprocess
import json
from pathlib import Path
from typing import Dict, List, Tuple


class MdBookLinkTester:
    """Tests mdBook links and structure."""
    
    def __init__(self, src_dir: str = "src"):
        self.src_dir = Path(src_dir)
        self.test_results = {
            "structure_test": {"passed": False, "details": []},
            "link_validation": {"passed": False, "details": []},
            "build_test": {"passed": False, "details": []},
            "navigation_test": {"passed": False, "details": []},
            "cross_reference_test": {"passed": False, "details": []}
        }
    
    def run_all_tests(self) -> bool:
        """Run all tests and return overall success."""
        print("Running comprehensive mdBook link tests...")
        print("=" * 60)
        
        # Test 1: Structure validation
        self.test_structure()
        
        # Test 2: Link validation
        self.test_links()
        
        # Test 3: Build test
        self.test_build()
        
        # Test 4: Navigation test
        self.test_navigation()
        
        # Test 5: Cross-reference test
        self.test_cross_references()
        
        # Generate report
        self.generate_report()
        
        # Return overall success
        return all(test["passed"] for test in self.test_results.values())
    
    def test_structure(self) -> None:
        """Test that the mdBook structure is correct."""
        print("Testing mdBook structure...")
        
        required_files = [
            "SUMMARY.md",
            "introduction.md",
            "quick-reference/README.md",
            "environment-setup/README.md",
            "core-concepts/README.md",
            "embedded-patterns/README.md",
            "cryptography/README.md",
            "migration/README.md"
        ]
        
        missing_files = []
        for file_path in required_files:
            if not (self.src_dir / file_path).exists():
                missing_files.append(file_path)
        
        if missing_files:
            self.test_results["structure_test"]["passed"] = False
            self.test_results["structure_test"]["details"] = [
                f"Missing required files: {', '.join(missing_files)}"
            ]
            print(f"  ❌ Structure test failed: {len(missing_files)} missing files")
        else:
            self.test_results["structure_test"]["passed"] = True
            self.test_results["structure_test"]["details"] = [
                f"All {len(required_files)} required files present"
            ]
            print(f"  ✅ Structure test passed: All required files present")
    
    def test_links(self) -> None:
        """Test that all links are valid."""
        print("Testing link validation...")
        
        try:
            # Run link validator
            result = subprocess.run(
                ["python3", "link_validator.py"],
                capture_output=True,
                text=True,
                timeout=30
            )
            
            if result.returncode == 0:
                # Parse output to get validation results
                output_lines = result.stdout.split('\n')
                success_rate_line = [line for line in output_lines if "Success rate:" in line]
                
                if success_rate_line and "100.0%" in success_rate_line[0]:
                    self.test_results["link_validation"]["passed"] = True
                    self.test_results["link_validation"]["details"] = [
                        "All links validated successfully (100% success rate)"
                    ]
                    print("  ✅ Link validation passed: 100% success rate")
                else:
                    self.test_results["link_validation"]["passed"] = False
                    self.test_results["link_validation"]["details"] = [
                        "Link validation failed - not all links are valid"
                    ]
                    print("  ❌ Link validation failed: Some links are broken")
            else:
                self.test_results["link_validation"]["passed"] = False
                self.test_results["link_validation"]["details"] = [
                    f"Link validator failed with exit code {result.returncode}"
                ]
                print(f"  ❌ Link validation failed: Exit code {result.returncode}")
                
        except Exception as e:
            self.test_results["link_validation"]["passed"] = False
            self.test_results["link_validation"]["details"] = [f"Error running link validator: {e}"]
            print(f"  ❌ Link validation error: {e}")
    
    def test_build(self) -> None:
        """Test that mdBook can build successfully."""
        print("Testing mdBook build...")
        
        try:
            # Check if mdbook is available
            mdbook_check = subprocess.run(
                ["mdbook", "--version"],
                capture_output=True,
                text=True,
                timeout=10
            )
            
            if mdbook_check.returncode != 0:
                self.test_results["build_test"]["passed"] = False
                self.test_results["build_test"]["details"] = [
                    "mdbook command not available - skipping build test"
                ]
                print("  ⚠️  Build test skipped: mdbook not available")
                return
            
            # Try to build the book
            build_result = subprocess.run(
                ["mdbook", "build"],
                capture_output=True,
                text=True,
                timeout=60
            )
            
            if build_result.returncode == 0:
                self.test_results["build_test"]["passed"] = True
                self.test_results["build_test"]["details"] = [
                    "mdBook built successfully"
                ]
                print("  ✅ Build test passed: mdBook built successfully")
            else:
                self.test_results["build_test"]["passed"] = False
                self.test_results["build_test"]["details"] = [
                    f"mdBook build failed: {build_result.stderr}"
                ]
                print("  ❌ Build test failed: mdBook build error")
                
        except subprocess.TimeoutExpired:
            self.test_results["build_test"]["passed"] = False
            self.test_results["build_test"]["details"] = ["Build test timed out"]
            print("  ❌ Build test failed: Timeout")
        except Exception as e:
            self.test_results["build_test"]["passed"] = False
            self.test_results["build_test"]["details"] = [f"Build test error: {e}"]
            print(f"  ❌ Build test error: {e}")
    
    def test_navigation(self) -> None:
        """Test that navigation structure is correct."""
        print("Testing navigation structure...")
        
        try:
            with open(self.src_dir / "SUMMARY.md", 'r', encoding='utf-8') as f:
                summary_content = f.read()
            
            # Check for required navigation elements
            required_sections = [
                "Quick Reference",
                "Environment Setup", 
                "Core Language Concepts",
                "Embedded-Specific Patterns",
                "Cryptography Implementation",
                "Migration and Integration"
            ]
            
            missing_sections = []
            for section in required_sections:
                if section not in summary_content:
                    missing_sections.append(section)
            
            # Check for proper link format
            import re
            link_pattern = re.compile(r'\[([^\]]+)\]\(([^)]+)\)')
            links = link_pattern.findall(summary_content)
            
            if missing_sections:
                self.test_results["navigation_test"]["passed"] = False
                self.test_results["navigation_test"]["details"] = [
                    f"Missing navigation sections: {', '.join(missing_sections)}"
                ]
                print(f"  ❌ Navigation test failed: Missing sections")
            elif len(links) < 30:  # Should have many navigation links
                self.test_results["navigation_test"]["passed"] = False
                self.test_results["navigation_test"]["details"] = [
                    f"Insufficient navigation links: {len(links)} found"
                ]
                print(f"  ❌ Navigation test failed: Too few links")
            else:
                self.test_results["navigation_test"]["passed"] = True
                self.test_results["navigation_test"]["details"] = [
                    f"Navigation structure correct: {len(links)} links, all sections present"
                ]
                print(f"  ✅ Navigation test passed: Complete structure with {len(links)} links")
                
        except Exception as e:
            self.test_results["navigation_test"]["passed"] = False
            self.test_results["navigation_test"]["details"] = [f"Navigation test error: {e}"]
            print(f"  ❌ Navigation test error: {e}")
    
    def test_cross_references(self) -> None:
        """Test that cross-references between chapters work."""
        print("Testing cross-references...")
        
        try:
            cross_ref_count = 0
            cross_ref_files = 0
            
            # Check for cross-references in key files
            test_files = [
                "core-concepts/functional.md",
                "core-concepts/advanced-types.md",
                "cryptography/secure-patterns.md",
                "migration/ffi-integration.md"
            ]
            
            import re
            cross_ref_pattern = re.compile(r'\[([^\]]+)\]\(\.\.\/([^)]+)\)')
            
            for file_path in test_files:
                full_path = self.src_dir / file_path
                if full_path.exists():
                    with open(full_path, 'r', encoding='utf-8') as f:
                        content = f.read()
                    
                    cross_refs = cross_ref_pattern.findall(content)
                    if cross_refs:
                        cross_ref_files += 1
                        cross_ref_count += len(cross_refs)
            
            if cross_ref_count > 0:
                self.test_results["cross_reference_test"]["passed"] = True
                self.test_results["cross_reference_test"]["details"] = [
                    f"Found {cross_ref_count} cross-references in {cross_ref_files} files"
                ]
                print(f"  ✅ Cross-reference test passed: {cross_ref_count} cross-references found")
            else:
                self.test_results["cross_reference_test"]["passed"] = False
                self.test_results["cross_reference_test"]["details"] = [
                    "No cross-references found between chapters"
                ]
                print("  ❌ Cross-reference test failed: No cross-references found")
                
        except Exception as e:
            self.test_results["cross_reference_test"]["passed"] = False
            self.test_results["cross_reference_test"]["details"] = [f"Cross-reference test error: {e}"]
            print(f"  ❌ Cross-reference test error: {e}")
    
    def generate_report(self) -> None:
        """Generate a comprehensive test report."""
        print("\n" + "=" * 60)
        print("TEST RESULTS SUMMARY")
        print("=" * 60)
        
        total_tests = len(self.test_results)
        passed_tests = sum(1 for test in self.test_results.values() if test["passed"])
        
        print(f"Overall: {passed_tests}/{total_tests} tests passed")
        print()
        
        for test_name, result in self.test_results.items():
            status = "✅ PASS" if result["passed"] else "❌ FAIL"
            print(f"{status} {test_name.replace('_', ' ').title()}")
            for detail in result["details"]:
                print(f"    {detail}")
            print()
        
        # Save detailed report
        with open("mdbook_test_report.json", 'w', encoding='utf-8') as f:
            json.dump(self.test_results, f, indent=2)
        
        print("Detailed report saved to mdbook_test_report.json")
        print("=" * 60)


def main():
    """Main function to run all tests."""
    tester = MdBookLinkTester()
    success = tester.run_all_tests()
    
    if success:
        print("\n🎉 All tests passed! mdBook structure is ready.")
        exit(0)
    else:
        print("\n⚠️  Some tests failed. Please review the issues above.")
        exit(1)


if __name__ == "__main__":
    main()