// BioMiner Indexd Documentation JavaScript

document.addEventListener('DOMContentLoaded', function() {
    // Initialize all components
    initAnimations();
    initSmoothScrolling();
    initActiveNavigation();
    initCodeHighlighting();
    initTooltips();
    initScrollEffects();
});

// Animation initialization
function initAnimations() {
    // Add fade-in animation to elements when they come into view
    const observerOptions = {
        threshold: 0.1,
        rootMargin: '0px 0px -50px 0px'
    };

    const observer = new IntersectionObserver((entries) => {
        entries.forEach(entry => {
            if (entry.isIntersecting) {
                entry.target.classList.add('fade-in-up');
                observer.unobserve(entry.target);
            }
        });
    }, observerOptions);

    // Observe elements for animation
    const animateElements = document.querySelectorAll('.feature-card, .tech-card, .capability-card, .practice-card, .resource-card, .repo-card, .contribution-area, .guideline-card, .dataset-card');
    animateElements.forEach(el => observer.observe(el));
}

// Smooth scrolling for anchor links
function initSmoothScrolling() {
    const links = document.querySelectorAll('a[href^="#"]');
    
    links.forEach(link => {
        link.addEventListener('click', function(e) {
            e.preventDefault();
            
            const targetId = this.getAttribute('href');
            const targetElement = document.querySelector(targetId);
            
            if (targetElement) {
                const offsetTop = targetElement.offsetTop - 80; // Account for fixed navbar
                
                window.scrollTo({
                    top: offsetTop,
                    behavior: 'smooth'
                });
            }
        });
    });
}

// Active navigation highlighting
function initActiveNavigation() {
    const sections = document.querySelectorAll('section[id]');
    const navLinks = document.querySelectorAll('.navbar-nav .nav-link');
    
    window.addEventListener('scroll', () => {
        let current = '';
        
        sections.forEach(section => {
            const sectionTop = section.offsetTop;
            const sectionHeight = section.clientHeight;
            
            if (window.pageYOffset >= sectionTop - 100) {
                current = section.getAttribute('id');
            }
        });
        
        navLinks.forEach(link => {
            link.classList.remove('active');
            if (link.getAttribute('href') === `#${current}`) {
                link.classList.add('active');
            }
        });
    });
}

// Code highlighting
function initCodeHighlighting() {
    const codeBlocks = document.querySelectorAll('pre code');
    
    codeBlocks.forEach(block => {
        // Add copy button
        const copyButton = document.createElement('button');
        copyButton.className = 'btn btn-sm btn-outline-secondary copy-btn';
        copyButton.innerHTML = '<i class="fas fa-copy"></i> Copy';
        copyButton.style.position = 'absolute';
        copyButton.style.top = '0.5rem';
        copyButton.style.right = '0.5rem';
        
        copyButton.addEventListener('click', () => {
            navigator.clipboard.writeText(block.textContent).then(() => {
                copyButton.innerHTML = '<i class="fas fa-check"></i> Copied!';
                setTimeout(() => {
                    copyButton.innerHTML = '<i class="fas fa-copy"></i> Copy';
                }, 2000);
            });
        });
        
        // Make code block container relative for absolute positioning
        const container = block.closest('.code-block');
        if (container) {
            container.style.position = 'relative';
            container.appendChild(copyButton);
        }
    });
}

// Tooltips initialization
function initTooltips() {
    // Initialize Bootstrap tooltips if available
    if (typeof bootstrap !== 'undefined' && bootstrap.Tooltip) {
        const tooltipTriggerList = [].slice.call(document.querySelectorAll('[data-bs-toggle="tooltip"]'));
        tooltipTriggerList.map(function (tooltipTriggerEl) {
            return new bootstrap.Tooltip(tooltipTriggerEl);
        });
    }
}

// Scroll effects
function initScrollEffects() {
    // Parallax effect for hero sections
    const heroSections = document.querySelectorAll('.hero-section, .hero-section-small');
    
    window.addEventListener('scroll', () => {
        const scrolled = window.pageYOffset;
        
        heroSections.forEach(section => {
            const rate = scrolled * -0.5;
            section.style.transform = `translateY(${rate}px)`;
        });
    });
    
    // Navbar background change on scroll
    const navbar = document.querySelector('.navbar');
    
    window.addEventListener('scroll', () => {
        if (window.scrollY > 50) {
            navbar.classList.add('navbar-scrolled');
        } else {
            navbar.classList.remove('navbar-scrolled');
        }
    });
}

// Dataset card interactions
function initDatasetInteractions() {
    const datasetCards = document.querySelectorAll('.dataset-card');
    
    datasetCards.forEach(card => {
        card.addEventListener('mouseenter', function() {
            this.style.transform = 'translateY(-10px) scale(1.02)';
        });
        
        card.addEventListener('mouseleave', function() {
            this.style.transform = 'translateY(0) scale(1)';
        });
    });
}

// Copilot chat simulation
function initCopilotSimulation() {
    const chatInput = document.querySelector('.chat-input input');
    const sendButton = document.querySelector('.chat-input button');
    
    if (chatInput && sendButton) {
        // Simulate typing effect
        const messages = [
            "What datasets contain RNA-seq data for breast cancer?",
            "How do I download the clinical data?",
            "Show me the mutation analysis results",
            "What's the age distribution in this dataset?"
        ];
        
        let currentMessageIndex = 0;
        
        function simulateTyping() {
            const message = messages[currentMessageIndex];
            let charIndex = 0;
            
            chatInput.value = '';
            chatInput.disabled = false;
            
            const typeInterval = setInterval(() => {
                chatInput.value += message[charIndex];
                charIndex++;
                
                if (charIndex >= message.length) {
                    clearInterval(typeInterval);
                    setTimeout(() => {
                        sendButton.disabled = false;
                    }, 500);
                }
            }, 100);
        }
        
        // Auto-simulate typing every 10 seconds
        setInterval(() => {
            simulateTyping();
            currentMessageIndex = (currentMessageIndex + 1) % messages.length;
        }, 10000);
        
        // Initial simulation
        setTimeout(simulateTyping, 2000);
    }
}

// Repository stats animation
function initRepoStatsAnimation() {
    const stats = document.querySelectorAll('.stat span');
    
    const observer = new IntersectionObserver((entries) => {
        entries.forEach(entry => {
            if (entry.isIntersecting) {
                animateNumber(entry.target);
                observer.unobserve(entry.target);
            }
        });
    });
    
    stats.forEach(stat => observer.observe(stat));
}

function animateNumber(element) {
    const target = parseInt(element.textContent) || Math.floor(Math.random() * 1000) + 100;
    let current = 0;
    const increment = target / 50;
    
    const timer = setInterval(() => {
        current += increment;
        if (current >= target) {
            current = target;
            clearInterval(timer);
        }
        element.textContent = Math.floor(current).toLocaleString();
    }, 50);
}

// Search functionality
function initSearch() {
    const searchInput = document.querySelector('.search-input');
    const searchResults = document.querySelector('.search-results');
    
    if (searchInput && searchResults) {
        searchInput.addEventListener('input', (e) => {
            const query = e.target.value.toLowerCase();
            
            if (query.length < 2) {
                searchResults.style.display = 'none';
                return;
            }
            
            // Simulate search results
            const results = [
                { title: 'Dataset Building Guide', url: 'datasets.html#guide' },
                { title: 'API Documentation', url: 'features.html#api' },
                { title: 'Quick Start Guide', url: 'features.html#quick-start' },
                { title: 'Contributing Guidelines', url: 'github.html#contributing' }
            ].filter(item => item.title.toLowerCase().includes(query));
            
            displaySearchResults(results);
        });
    }
}

function displaySearchResults(results) {
    const searchResults = document.querySelector('.search-results');
    
    if (results.length === 0) {
        searchResults.innerHTML = '<div class="p-3 text-muted">No results found</div>';
    } else {
        searchResults.innerHTML = results.map(result => 
            `<a href="${result.url}" class="dropdown-item">${result.title}</a>`
        ).join('');
    }
    
    searchResults.style.display = 'block';
}

// Theme toggle (if needed)
function initThemeToggle() {
    const themeToggle = document.querySelector('.theme-toggle');
    
    if (themeToggle) {
        themeToggle.addEventListener('click', () => {
            document.body.classList.toggle('dark-theme');
            localStorage.setItem('theme', document.body.classList.contains('dark-theme') ? 'dark' : 'light');
        });
        
        // Load saved theme
        const savedTheme = localStorage.getItem('theme');
        if (savedTheme === 'dark') {
            document.body.classList.add('dark-theme');
        }
    }
}

// Mobile menu improvements
function initMobileMenu() {
    const navbarToggler = document.querySelector('.navbar-toggler');
    const navbarCollapse = document.querySelector('.navbar-collapse');
    
    if (navbarToggler && navbarCollapse) {
        // Close mobile menu when clicking on a link
        const navLinks = navbarCollapse.querySelectorAll('.nav-link');
        
        navLinks.forEach(link => {
            link.addEventListener('click', () => {
                if (window.innerWidth < 992) {
                    navbarCollapse.classList.remove('show');
                }
            });
        });
        
        // Close mobile menu when clicking outside
        document.addEventListener('click', (e) => {
            if (!navbarCollapse.contains(e.target) && !navbarToggler.contains(e.target)) {
                navbarCollapse.classList.remove('show');
            }
        });
    }
}

// Performance optimization
function initPerformanceOptimizations() {
    // Lazy load images
    const images = document.querySelectorAll('img[data-src]');
    
    const imageObserver = new IntersectionObserver((entries) => {
        entries.forEach(entry => {
            if (entry.isIntersecting) {
                const img = entry.target;
                img.src = img.dataset.src;
                img.classList.remove('lazy');
                imageObserver.unobserve(img);
            }
        });
    });
    
    images.forEach(img => imageObserver.observe(img));
    
    // Debounce scroll events
    let scrollTimeout;
    window.addEventListener('scroll', () => {
        clearTimeout(scrollTimeout);
        scrollTimeout = setTimeout(() => {
            // Handle scroll-based operations here
        }, 100);
    });
}

// Error handling
function initErrorHandling() {
    window.addEventListener('error', (e) => {
        console.error('JavaScript error:', e.error);
        // You could send this to an error tracking service
    });
    
    window.addEventListener('unhandledrejection', (e) => {
        console.error('Unhandled promise rejection:', e.reason);
        // You could send this to an error tracking service
    });
}

// Initialize all features
document.addEventListener('DOMContentLoaded', function() {
    initDatasetInteractions();
    initCopilotSimulation();
    initRepoStatsAnimation();
    initSearch();
    initThemeToggle();
    initMobileMenu();
    initPerformanceOptimizations();
    initErrorHandling();
});

// Export functions for potential external use
window.BioMinerDocs = {
    initAnimations,
    initSmoothScrolling,
    initActiveNavigation,
    initCodeHighlighting,
    initTooltips,
    initScrollEffects,
    initDatasetInteractions,
    initCopilotSimulation,
    initRepoStatsAnimation,
    initSearch,
    initThemeToggle,
    initMobileMenu,
    initPerformanceOptimizations,
    initErrorHandling
}; 