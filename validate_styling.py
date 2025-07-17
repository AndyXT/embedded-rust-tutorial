#!/usr/bin/env python3
"""
Validate that all styling enhancements for task 15 are properly implemented.
"""

import os
import re
import json
from pathlib import Path

def validate_css_enhancements():
    """Validate that all required CSS enhancements are present."""
    css_file = Path("theme/custom.css")
    
    if not css_file.exists():
        return False, "Custom CSS file not found"
    
    css_content = css_file.read_text()
    
    required_features = [
        # Table formatting improvements
        r"\.table-wrapper",
        r"table\.comparison",
        r"table caption",
        r"position: sticky",  # Sticky headers
        
        # Responsive design
        r"@media screen and \(max-width: 768px\)",
        r"@media screen and \(max-width: 480px\)",
        r"transform: translateX",  # Mobile sidebar
        r"min-height: 44px",  # Touch targets
        
        # Print-friendly styling
        r"@media print",
        r"page-break-inside: avoid",
        r"page-break-after: avoid",
        r"@page",  # Page settings
        
        # Code block enhancements
        r"\.hljs",
        r"font-family.*Source Code Pro",
        r"overflow-x: auto",
        
        # Search enhancements
        r"#searchresults",
        r"\.searchresult",
        
        # Accessibility features
        r"\.sr-only",
        r"outline.*focus",
        r"\.skip-link",
        
        # Technical content styling
        r"kbd\s*{",
        r"blockquote",
        r"\.callout",
        r"\.code-comparison"
    ]
    
    missing_features = []
    for feature in required_features:
        if not re.search(feature, css_content, re.IGNORECASE):
            missing_features.append(feature)
    
    if missing_features:
        return False, f"Missing CSS features: {missing_features}"
    
    return True, "All CSS enhancements present"

def validate_js_enhancements():
    """Validate that all required JavaScript enhancements are present."""
    js_file = Path("theme/custom.js")
    
    if not js_file.exists():
        return False, "Custom JavaScript file not found"
    
    js_content = js_file.read_text()
    
    required_features = [
        # Copy functionality
        r"initializeCopyButtons",
        r"copyToClipboard",
        r"navigator\.clipboard",
        
        # Responsive features
        r"initializeResponsiveFeatures",
        r"mobile-nav-toggle",
        r"sidebar-visible",
        
        # Search enhancements
        r"initializeSearchEnhancements",
        r"navigateSearchResults",
        r"addSearchShortcuts",
        r"showSearchHelp",
        
        # Print optimizations
        r"initializePrintOptimizations",
        r"beforeprint",
        r"afterprint",
        
        # Accessibility features
        r"ArrowDown.*ArrowUp",  # Keyboard navigation
        r"Escape",  # Escape key handling
        r"Ctrl.*K",  # Search shortcut
    ]
    
    missing_features = []
    for feature in required_features:
        if not re.search(feature, js_content, re.IGNORECASE):
            missing_features.append(feature)
    
    if missing_features:
        return False, f"Missing JavaScript features: {missing_features}"
    
    return True, "All JavaScript enhancements present"

def validate_book_config():
    """Validate that book.toml has proper search optimization."""
    config_file = Path("book.toml")
    
    if not config_file.exists():
        return False, "book.toml not found"
    
    config_content = config_file.read_text()
    
    required_settings = [
        r"additional-css.*custom\.css",
        r"additional-js.*custom\.js",
        r"limit-results.*50",  # Increased search results
        r"boost-title.*3",  # Enhanced title boosting
        r"heading-split-level.*2",  # Better search indexing
    ]
    
    missing_settings = []
    for setting in required_settings:
        if not re.search(setting, config_content, re.IGNORECASE):
            missing_settings.append(setting)
    
    if missing_settings:
        return False, f"Missing book.toml settings: {missing_settings}"
    
    return True, "Book configuration properly optimized"

def validate_build_output():
    """Validate that the build output includes all enhancements."""
    book_dir = Path("book")
    
    if not book_dir.exists():
        return False, "Book directory not found - run mdbook build first"
    
    # Check that custom files are copied
    custom_css = book_dir / "theme" / "custom.css"
    custom_js = book_dir / "theme" / "custom.js"
    
    if not custom_css.exists():
        return False, "Custom CSS not copied to book output"
    
    if not custom_js.exists():
        return False, "Custom JavaScript not copied to book output"
    
    # Check that HTML includes custom files
    index_html = book_dir / "index.html"
    if index_html.exists():
        html_content = index_html.read_text()
        
        if "theme/custom.css" not in html_content:
            return False, "Custom CSS not included in HTML"
        
        if "theme/custom.js" not in html_content:
            return False, "Custom JavaScript not included in HTML"
    
    return True, "Build output properly includes all enhancements"

def main():
    """Run all validation checks."""
    print("Validating Task 15: Custom Styling and Enhancements")
    print("=" * 60)
    
    checks = [
        ("CSS Enhancements", validate_css_enhancements),
        ("JavaScript Enhancements", validate_js_enhancements),
        ("Book Configuration", validate_book_config),
        ("Build Output", validate_build_output),
    ]
    
    results = {}
    all_passed = True
    
    for check_name, check_func in checks:
        try:
            success, message = check_func()
            results[check_name] = {"success": success, "message": message}
            
            status = "✅ PASS" if success else "❌ FAIL"
            print(f"{status} {check_name}: {message}")
            
            if not success:
                all_passed = False
                
        except Exception as e:
            results[check_name] = {"success": False, "message": f"Error: {str(e)}"}
            print(f"❌ FAIL {check_name}: Error: {str(e)}")
            all_passed = False
    
    print("\n" + "=" * 60)
    
    if all_passed:
        print("🎉 All styling enhancements successfully implemented!")
        print("\nTask 15 Requirements Validation:")
        print("✅ Custom CSS for improved table formatting and code block styling")
        print("✅ Responsive design improvements for mobile devices")
        print("✅ Search functionality optimization for technical content")
        print("✅ Print-friendly styling for offline reference use")
    else:
        print("⚠️  Some styling enhancements need attention.")
    
    # Save detailed results
    with open("task15_validation_results.json", "w") as f:
        json.dump(results, f, indent=2)
    
    return all_passed

if __name__ == "__main__":
    success = main()
    exit(0 if success else 1)