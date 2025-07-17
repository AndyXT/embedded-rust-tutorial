// Custom JavaScript for Embedded Rust Tutorial

(function() {
    'use strict';

    // Initialize when DOM is loaded
    document.addEventListener('DOMContentLoaded', function() {
        initializeCopyButtons();
        initializeResponsiveFeatures();
        initializeSearchEnhancements();
        initializePrintOptimizations();
        initializeCompilationStatusFeatures();
        initializePlaygroundEnhancements();
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

    // Initialize compilation status features
    function initializeCompilationStatusFeatures() {
        // Add interactive features to compilation status indicators
        const statusIndicators = document.querySelectorAll('.compilation-status');
        
        statusIndicators.forEach(function(indicator) {
            // Add click handler for detailed error information
            if (indicator.classList.contains('compilation-failed')) {
                indicator.style.cursor = 'pointer';
                indicator.addEventListener('click', function() {
                    showCompilationErrorDetails(indicator);
                });
            }
            
            // Add hover effects for better UX
            indicator.addEventListener('mouseenter', function() {
                this.style.transform = 'translateY(-1px)';
                this.style.boxShadow = '0 2px 4px rgba(0, 0, 0, 0.1)';
            });
            
            indicator.addEventListener('mouseleave', function() {
                this.style.transform = 'translateY(0)';
                this.style.boxShadow = 'none';
            });
        });

        // Add context indicators to code blocks
        addCodeBlockContextIndicators();
        
        // Initialize compilation metrics display
        initializeCompilationMetrics();
    }

    // Show detailed compilation error information
    function showCompilationErrorDetails(indicator) {
        const errorTitle = indicator.getAttribute('title');
        if (!errorTitle) return;

        const errorModal = document.createElement('div');
        errorModal.className = 'compilation-error-modal';
        errorModal.innerHTML = `
            <div class="error-modal-content">
                <div class="error-modal-header">
                    <h3>Compilation Error Details</h3>
                    <button class="error-modal-close" aria-label="Close error details">&times;</button>
                </div>
                <div class="error-modal-body">
                    <pre><code>${escapeHtml(errorTitle)}</code></pre>
                </div>
                <div class="error-modal-footer">
                    <button class="error-modal-copy">Copy Error</button>
                    <button class="error-modal-close-btn">Close</button>
                </div>
            </div>
        `;

        // Style the modal
        errorModal.style.cssText = `
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

        const modalContent = errorModal.querySelector('.error-modal-content');
        modalContent.style.cssText = `
            background: var(--bg);
            border: 1px solid var(--theme-popup-border);
            border-radius: 8px;
            max-width: 600px;
            max-height: 80vh;
            overflow-y: auto;
            color: var(--fg);
            box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
        `;

        // Add event handlers
        const closeButtons = errorModal.querySelectorAll('.error-modal-close, .error-modal-close-btn');
        closeButtons.forEach(button => {
            button.addEventListener('click', () => errorModal.remove());
        });

        const copyButton = errorModal.querySelector('.error-modal-copy');
        copyButton.addEventListener('click', () => {
            copyToClipboard(errorTitle, copyButton);
        });

        // Close on click outside
        errorModal.addEventListener('click', function(e) {
            if (e.target === errorModal) {
                errorModal.remove();
            }
        });

        document.body.appendChild(errorModal);
    }

    // Add context indicators to code blocks
    function addCodeBlockContextIndicators() {
        const codeBlocks = document.querySelectorAll('pre code');
        
        codeBlocks.forEach(function(codeBlock) {
            const pre = codeBlock.parentElement;
            const codeText = codeBlock.textContent;
            
            // Detect context from code content
            const context = detectCodeContext(codeText);
            
            if (context) {
                pre.classList.add(`code-block-${context}`);
                
                // Add target architecture indicator if embedded
                if (context === 'embedded' || context === 'no-std') {
                    addTargetIndicator(pre, detectTargetArchitecture(codeText));
                }
                
                // Add feature flags if detected
                const features = detectFeatureFlags(codeText);
                if (features.length > 0) {
                    addFeatureFlags(pre, features);
                }
            }
        });
    }

    // Detect code context from content
    function detectCodeContext(code) {
        if (code.includes('#![no_std]') || code.includes('#![no_main]')) {
            return 'no-std';
        }
        if (code.includes('cortex_m') || code.includes('embedded_hal') || code.includes('stm32')) {
            return 'hardware';
        }
        if (code.includes('zeroize') || code.includes('crypto') || code.includes('aes') || code.includes('sha')) {
            return 'crypto';
        }
        if (code.includes('// snippet') || code.includes('// ...')) {
            return 'snippet';
        }
        return null;
    }

    // Detect target architecture
    function detectTargetArchitecture(code) {
        if (code.includes('thumbv7em') || code.includes('cortex-m')) {
            return 'thumbv7em';
        }
        if (code.includes('x86_64')) {
            return 'x86_64';
        }
        return 'generic';
    }

    // Detect feature flags
    function detectFeatureFlags(code) {
        const features = [];
        if (code.includes('crypto') || code.includes('zeroize')) {
            features.push('crypto');
        }
        if (code.includes('embedded_hal') || code.includes('cortex_m')) {
            features.push('hardware');
        }
        if (code.includes('#![no_std]')) {
            features.push('embedded');
        }
        return features;
    }

    // Add target architecture indicator
    function addTargetIndicator(pre, target) {
        const indicator = document.createElement('span');
        indicator.className = `target-indicator ${target}`;
        indicator.textContent = target;
        indicator.style.cssText = `
            position: absolute;
            bottom: 8px;
            right: 8px;
            z-index: 10;
        `;
        pre.style.position = 'relative';
        pre.appendChild(indicator);
    }

    // Add feature flags display
    function addFeatureFlags(pre, features) {
        const flagsContainer = document.createElement('div');
        flagsContainer.className = 'feature-flags';
        flagsContainer.style.cssText = `
            position: absolute;
            bottom: 8px;
            left: 8px;
            z-index: 10;
        `;

        features.forEach(feature => {
            const flag = document.createElement('span');
            flag.className = `feature-flag ${feature}`;
            flag.textContent = feature;
            flagsContainer.appendChild(flag);
        });

        pre.style.position = 'relative';
        pre.appendChild(flagsContainer);
    }

    // Initialize compilation metrics display
    function initializeCompilationMetrics() {
        const statusIndicators = document.querySelectorAll('.compilation-status');
        let totalExamples = statusIndicators.length;
        let successfulExamples = document.querySelectorAll('.compilation-status.compilation-success').length;
        let failedExamples = document.querySelectorAll('.compilation-status.compilation-failed').length;

        if (totalExamples > 0) {
            addCompilationSummary(totalExamples, successfulExamples, failedExamples);
        }
    }

    // Add compilation summary to page
    function addCompilationSummary(total, successful, failed) {
        const summary = document.createElement('div');
        summary.className = 'compilation-summary';
        summary.innerHTML = `
            <div class="summary-header">
                <h4>Code Example Compilation Status</h4>
            </div>
            <div class="summary-stats">
                <div class="stat-item">
                    <span class="stat-number">${total}</span>
                    <span class="stat-label">Total Examples</span>
                </div>
                <div class="stat-item success">
                    <span class="stat-number">${successful}</span>
                    <span class="stat-label">✓ Compiled</span>
                </div>
                <div class="stat-item failed">
                    <span class="stat-number">${failed}</span>
                    <span class="stat-label">✗ Failed</span>
                </div>
                <div class="stat-item">
                    <span class="stat-number">${Math.round((successful / total) * 100)}%</span>
                    <span class="stat-label">Success Rate</span>
                </div>
            </div>
        `;

        // Style the summary
        summary.style.cssText = `
            background: var(--theme-hover);
            border: 1px solid var(--theme-popup-border);
            border-radius: 6px;
            padding: 16px;
            margin: 16px 0;
            font-size: 14px;
        `;

        // Insert at the top of content
        const content = document.querySelector('.content');
        if (content) {
            const firstChild = content.firstElementChild;
            if (firstChild) {
                content.insertBefore(summary, firstChild);
            } else {
                content.appendChild(summary);
            }
        }
    }

    // Initialize playground enhancements
    function initializePlaygroundEnhancements() {
        // Process playground configurations
        const playgroundConfigs = document.querySelectorAll('.playground-config');
        
        playgroundConfigs.forEach(function(config) {
            try {
                const configData = JSON.parse(config.textContent);
                enhancePlaygroundWithConfig(config, configData);
            } catch (e) {
                console.warn('Failed to parse playground config:', e);
            }
        });

        // Add run buttons to runnable examples
        addRunButtons();
        
        // Initialize playground features
        initializePlaygroundFeatures();
    }

    // Enhance playground with configuration
    function enhancePlaygroundWithConfig(configElement, config) {
        const codeBlock = configElement.previousElementSibling;
        if (codeBlock && codeBlock.tagName === 'PRE') {
            codeBlock.classList.add('playground-runnable');
            
            // Add configuration data as data attributes
            codeBlock.dataset.playgroundConfig = JSON.stringify(config);
            
            // Add edition indicator
            if (config.edition) {
                const editionIndicator = document.createElement('span');
                editionIndicator.className = 'edition-indicator';
                editionIndicator.textContent = `Rust ${config.edition}`;
                editionIndicator.style.cssText = `
                    position: absolute;
                    top: 8px;
                    left: 8px;
                    background: var(--links);
                    color: white;
                    padding: 2px 6px;
                    border-radius: 3px;
                    font-size: 10px;
                    font-weight: bold;
                    z-index: 10;
                `;
                codeBlock.style.position = 'relative';
                codeBlock.appendChild(editionIndicator);
            }
        }
    }

    // Add run buttons to runnable examples
    function addRunButtons() {
        const runnableBlocks = document.querySelectorAll('.playground-runnable');
        
        runnableBlocks.forEach(function(block) {
            const runButton = document.createElement('button');
            runButton.className = 'playground-run-button';
            runButton.textContent = '▶ Run';
            runButton.setAttribute('aria-label', 'Run code example');
            
            runButton.style.cssText = `
                position: absolute;
                top: 8px;
                right: 80px;
                background: #4caf50;
                color: white;
                border: none;
                padding: 4px 8px;
                border-radius: 3px;
                font-size: 11px;
                font-weight: bold;
                cursor: pointer;
                z-index: 10;
                transition: background-color 0.2s ease;
            `;
            
            runButton.addEventListener('click', function() {
                runCodeExample(block);
            });
            
            runButton.addEventListener('mouseenter', function() {
                this.style.backgroundColor = '#45a049';
            });
            
            runButton.addEventListener('mouseleave', function() {
                this.style.backgroundColor = '#4caf50';
            });
            
            block.style.position = 'relative';
            block.appendChild(runButton);
        });
    }

    // Run code example (placeholder - would integrate with actual playground)
    function runCodeExample(codeBlock) {
        const code = codeBlock.querySelector('code').textContent;
        const config = JSON.parse(codeBlock.dataset.playgroundConfig || '{}');
        
        // This would integrate with the Rust playground or local execution
        console.log('Running code example:', { code, config });
        
        // Show running indicator
        const runButton = codeBlock.querySelector('.playground-run-button');
        const originalText = runButton.textContent;
        runButton.textContent = '⏳ Running...';
        runButton.disabled = true;
        
        // Simulate execution (replace with actual playground integration)
        setTimeout(() => {
            runButton.textContent = originalText;
            runButton.disabled = false;
            showExecutionResult(codeBlock, 'Example executed successfully!');
        }, 2000);
    }

    // Show execution result
    function showExecutionResult(codeBlock, result) {
        const resultDiv = document.createElement('div');
        resultDiv.className = 'execution-result';
        resultDiv.innerHTML = `
            <div class="result-header">Output:</div>
            <pre><code>${escapeHtml(result)}</code></pre>
        `;
        
        resultDiv.style.cssText = `
            background: var(--theme-hover);
            border: 1px solid var(--theme-popup-border);
            border-top: none;
            border-radius: 0 0 6px 6px;
            padding: 12px;
            font-size: 13px;
            margin-top: 0;
        `;
        
        // Remove existing result if present
        const existingResult = codeBlock.nextElementSibling;
        if (existingResult && existingResult.classList.contains('execution-result')) {
            existingResult.remove();
        }
        
        codeBlock.parentNode.insertBefore(resultDiv, codeBlock.nextSibling);
        
        // Auto-remove after 10 seconds
        setTimeout(() => {
            if (resultDiv.parentNode) {
                resultDiv.remove();
            }
        }, 10000);
    }

    // Initialize additional playground features
    function initializePlaygroundFeatures() {
        // Add keyboard shortcuts for playground
        document.addEventListener('keydown', function(e) {
            // Ctrl/Cmd + Enter to run focused code block
            if ((e.ctrlKey || e.metaKey) && e.key === 'Enter') {
                const focusedElement = document.activeElement;
                const codeBlock = focusedElement.closest('.playground-runnable');
                if (codeBlock) {
                    e.preventDefault();
                    runCodeExample(codeBlock);
                }
            }
        });
        
        // Make code blocks focusable for keyboard navigation
        const runnableBlocks = document.querySelectorAll('.playground-runnable');
        runnableBlocks.forEach(function(block) {
            block.setAttribute('tabindex', '0');
            block.addEventListener('focus', function() {
                this.style.outline = '2px solid var(--links)';
                this.style.outlineOffset = '2px';
            });
            block.addEventListener('blur', function() {
                this.style.outline = 'none';
            });
        });
    }

    // Utility function to escape HTML
    function escapeHtml(text) {
        const div = document.createElement('div');
        div.textContent = text;
        return div.innerHTML;
    }

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