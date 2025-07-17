// Custom JavaScript for Embedded Rust Tutorial

(function() {
    'use strict';

    // Initialize when DOM is loaded
    document.addEventListener('DOMContentLoaded', function() {
        initializeCopyButtons();
        initializeResponsiveFeatures();
        initializeSearchEnhancements();
        initializePrintOptimizations();
    });

    // Copy-to-clipboard functionality for code blocks
    function initializeCopyButtons() {
        const codeBlocks = document.querySelectorAll('pre code');
        
        codeBlocks.forEach(function(codeBlock) {
            const pre = codeBlock.parentElement;
            
            // Create copy button
            const copyButton = document.createElement('button');
            copyButton.className = 'copy-button';
            copyButton.textContent = 'Copy';
            copyButton.setAttribute('aria-label', 'Copy code to clipboard');
            
            // Add click handler
            copyButton.addEventListener('click', function() {
                copyToClipboard(codeBlock.textContent, copyButton);
            });
            
            // Add button to code block
            pre.style.position = 'relative';
            pre.appendChild(copyButton);
        });
    }

    // Copy text to clipboard with visual feedback
    function copyToClipboard(text, button) {
        if (navigator.clipboard && window.isSecureContext) {
            // Modern clipboard API
            navigator.clipboard.writeText(text).then(function() {
                showCopySuccess(button);
            }).catch(function(err) {
                console.error('Failed to copy text: ', err);
                fallbackCopyToClipboard(text, button);
            });
        } else {
            // Fallback for older browsers
            fallbackCopyToClipboard(text, button);
        }
    }

    // Fallback copy method for older browsers
    function fallbackCopyToClipboard(text, button) {
        const textArea = document.createElement('textarea');
        textArea.value = text;
        textArea.style.position = 'fixed';
        textArea.style.left = '-999999px';
        textArea.style.top = '-999999px';
        document.body.appendChild(textArea);
        textArea.focus();
        textArea.select();
        
        try {
            document.execCommand('copy');
            showCopySuccess(button);
        } catch (err) {
            console.error('Fallback copy failed: ', err);
        }
        
        document.body.removeChild(textArea);
    }

    // Show visual feedback for successful copy
    function showCopySuccess(button) {
        const originalText = button.textContent;
        button.textContent = 'Copied!';
        button.classList.add('copied');
        
        setTimeout(function() {
            button.textContent = originalText;
            button.classList.remove('copied');
        }, 2000);
    }

    // Responsive navigation enhancements
    function initializeResponsiveFeatures() {
        // Mobile menu toggle enhancement
        const menuToggle = document.querySelector('.mobile-nav-toggle');
        const sidebar = document.querySelector('.sidebar');
        
        if (menuToggle && sidebar) {
            menuToggle.addEventListener('click', function() {
                sidebar.classList.toggle('sidebar-visible');
                document.body.classList.toggle('sidebar-visible');
            });
            
            // Close sidebar when clicking outside on mobile
            document.addEventListener('click', function(e) {
                if (window.innerWidth <= 768 && 
                    !sidebar.contains(e.target) && 
                    !menuToggle.contains(e.target) &&
                    sidebar.classList.contains('sidebar-visible')) {
                    sidebar.classList.remove('sidebar-visible');
                    document.body.classList.remove('sidebar-visible');
                }
            });
        }

        // Responsive table handling
        const tables = document.querySelectorAll('table');
        tables.forEach(function(table) {
            // Wrap tables for horizontal scrolling on mobile
            if (!table.parentElement.classList.contains('table-wrapper')) {
                const wrapper = document.createElement('div');
                wrapper.className = 'table-wrapper';
                table.parentElement.insertBefore(wrapper, table);
                wrapper.appendChild(table);
            }
        });

        // Handle window resize
        window.addEventListener('resize', function() {
            // Adjust layout based on screen size
            if (window.innerWidth > 768) {
                document.body.classList.remove('sidebar-visible');
                if (sidebar) {
                    sidebar.classList.remove('sidebar-visible');
                }
            }
        });
    }

    // Enhanced search functionality
    function initializeSearchEnhancements() {
        const searchInput = document.querySelector('#searchbar');
        const searchResults = document.querySelector('#searchresults');
        
        if (searchInput) {
            // Add keyboard navigation for search results
            searchInput.addEventListener('keydown', function(e) {
                if (e.key === 'ArrowDown' || e.key === 'ArrowUp') {
                    e.preventDefault();
                    navigateSearchResults(e.key === 'ArrowDown' ? 1 : -1);
                } else if (e.key === 'Enter') {
                    const activeResult = document.querySelector('.searchresult.active');
                    if (activeResult) {
                        const link = activeResult.querySelector('a');
                        if (link) {
                            link.click();
                        }
                    }
                } else if (e.key === 'Escape') {
                    // Clear search and close results
                    searchInput.value = '';
                    if (searchResults) {
                        searchResults.innerHTML = '';
                    }
                    searchInput.blur();
                }
            });

            // Clear search results when input is empty
            searchInput.addEventListener('input', function() {
                if (this.value.trim() === '') {
                    if (searchResults) {
                        searchResults.innerHTML = '';
                    }
                }
            });

            // Add search suggestions and technical content optimization
            let searchTimeout;
            searchInput.addEventListener('input', function() {
                clearTimeout(searchTimeout);
                const query = this.value.trim();
                
                if (query.length > 0) {
                    searchTimeout = setTimeout(() => {
                        enhanceSearchResults(query);
                    }, 300); // Debounce search
                }
            });

            // Add search shortcuts for technical terms
            addSearchShortcuts(searchInput);
        }
    }

    // Enhance search results for technical content
    function enhanceSearchResults(query) {
        // Add technical term suggestions
        const technicalTerms = {
            'ownership': ['borrow', 'lifetime', 'move', 'reference'],
            'memory': ['heap', 'stack', 'allocation', 'deallocation'],
            'crypto': ['encryption', 'hash', 'key', 'cipher', 'constant-time'],
            'embedded': ['no-std', 'interrupt', 'dma', 'hardware'],
            'error': ['result', 'option', 'panic', 'unwrap'],
            'async': ['future', 'await', 'tokio', 'executor']
        };

        // Check if query matches technical terms and suggest related terms
        const lowerQuery = query.toLowerCase();
        for (const [term, related] of Object.entries(technicalTerms)) {
            if (lowerQuery.includes(term) || related.some(r => lowerQuery.includes(r))) {
                // Could enhance search results here with related terms
                break;
            }
        }
    }

    // Add keyboard shortcuts for common searches
    function addSearchShortcuts(searchInput) {
        document.addEventListener('keydown', function(e) {
            // Ctrl/Cmd + K to focus search
            if ((e.ctrlKey || e.metaKey) && e.key === 'k') {
                e.preventDefault();
                searchInput.focus();
                searchInput.select();
            }
            
            // Ctrl/Cmd + / for search help
            if ((e.ctrlKey || e.metaKey) && e.key === '/') {
                e.preventDefault();
                showSearchHelp();
            }
        });
    }

    // Show search help overlay
    function showSearchHelp() {
        const helpOverlay = document.createElement('div');
        helpOverlay.className = 'search-help-overlay';
        helpOverlay.innerHTML = `
            <div class="search-help-content">
                <h3>Search Tips</h3>
                <ul>
                    <li><kbd>Ctrl+K</kbd> - Focus search</li>
                    <li><kbd>↑↓</kbd> - Navigate results</li>
                    <li><kbd>Enter</kbd> - Open result</li>
                    <li><kbd>Esc</kbd> - Clear search</li>
                </ul>
                <p>Try searching for: ownership, memory, crypto, embedded, error handling</p>
                <button onclick="this.parentElement.parentElement.remove()">Close</button>
            </div>
        `;
        
        helpOverlay.style.cssText = `
            position: fixed;
            top: 0;
            left: 0;
            right: 0;
            bottom: 0;
            background: rgba(0, 0, 0, 0.5);
            display: flex;
            align-items: center;
            justify-content: center;
            z-index: 1000;
        `;
        
        helpOverlay.querySelector('.search-help-content').style.cssText = `
            background: var(--bg);
            padding: 20px;
            border-radius: 8px;
            border: 1px solid var(--theme-popup-border);
            max-width: 400px;
            color: var(--fg);
        `;
        
        document.body.appendChild(helpOverlay);
        
        // Close on click outside
        helpOverlay.addEventListener('click', function(e) {
            if (e.target === helpOverlay) {
                helpOverlay.remove();
            }
        });
    }

    // Navigate search results with keyboard
    function navigateSearchResults(direction) {
        const results = document.querySelectorAll('.searchresult');
        const activeResult = document.querySelector('.searchresult.active');
        
        if (results.length === 0) return;
        
        let newIndex = 0;
        
        if (activeResult) {
            const currentIndex = Array.from(results).indexOf(activeResult);
            newIndex = currentIndex + direction;
            activeResult.classList.remove('active');
        }
        
        // Wrap around
        if (newIndex < 0) newIndex = results.length - 1;
        if (newIndex >= results.length) newIndex = 0;
        
        results[newIndex].classList.add('active');
        results[newIndex].scrollIntoView({ block: 'nearest' });
    }

    // Print optimization features
    function initializePrintOptimizations() {
        // Add print button functionality
        const printButton = document.createElement('button');
        printButton.textContent = 'Print';
        printButton.className = 'print-button';
        printButton.style.cssText = `
            position: fixed;
            top: 10px;
            right: 10px;
            z-index: 1000;
            padding: 8px 16px;
            background: var(--theme-popup-bg);
            border: 1px solid var(--theme-popup-border);
            border-radius: 4px;
            color: var(--fg);
            cursor: pointer;
            font-size: 14px;
        `;
        
        printButton.addEventListener('click', function() {
            window.print();
        });
        
        // Only show print button on larger screens
        if (window.innerWidth > 768) {
            document.body.appendChild(printButton);
        }

        // Optimize content for printing
        window.addEventListener('beforeprint', function() {
            // Expand all collapsed sections for printing
            const collapsedSections = document.querySelectorAll('.chapter:not(.expanded)');
            collapsedSections.forEach(function(section) {
                section.classList.add('expanded', 'print-expanded');
            });
        });

        window.addEventListener('afterprint', function() {
            // Restore collapsed state after printing
            const printExpanded = document.querySelectorAll('.print-expanded');
            printExpanded.forEach(function(section) {
                section.classList.remove('expanded', 'print-expanded');
            });
        });
    }

    // Add breadcrumb navigation
    function addBreadcrumbNavigation() {
        const content = document.querySelector('.content');
        const currentPath = window.location.pathname;
        
        if (content && currentPath !== '/') {
            const breadcrumb = document.createElement('nav');
            breadcrumb.className = 'breadcrumb';
            breadcrumb.setAttribute('aria-label', 'Breadcrumb navigation');
            
            // Build breadcrumb based on current path
            const pathParts = currentPath.split('/').filter(part => part);
            let breadcrumbHTML = '<a href="/">Home</a>';
            
            let currentUrl = '';
            pathParts.forEach(function(part, index) {
                currentUrl += '/' + part;
                const isLast = index === pathParts.length - 1;
                const displayName = part.replace(/-/g, ' ').replace(/\.html$/, '');
                const capitalizedName = displayName.charAt(0).toUpperCase() + displayName.slice(1);
                
                breadcrumbHTML += '<span class="separator">›</span>';
                
                if (isLast) {
                    breadcrumbHTML += '<span>' + capitalizedName + '</span>';
                } else {
                    breadcrumbHTML += '<a href="' + currentUrl + '">' + capitalizedName + '</a>';
                }
            });
            
            breadcrumb.innerHTML = breadcrumbHTML;
            content.insertBefore(breadcrumb, content.firstChild);
        }
    }

    // Initialize breadcrumb navigation
    document.addEventListener('DOMContentLoaded', function() {
        addBreadcrumbNavigation();
    });

    // Smooth scrolling for anchor links
    document.addEventListener('click', function(e) {
        if (e.target.tagName === 'A' && e.target.getAttribute('href').startsWith('#')) {
            e.preventDefault();
            const targetId = e.target.getAttribute('href').substring(1);
            const targetElement = document.getElementById(targetId);
            
            if (targetElement) {
                targetElement.scrollIntoView({
                    behavior: 'smooth',
                    block: 'start'
                });
                
                // Update URL without triggering page reload
                history.pushState(null, null, '#' + targetId);
            }
        }
    });

})();