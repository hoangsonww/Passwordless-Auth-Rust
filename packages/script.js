// Passwordless Auth Wiki - Interactive Script

// Theme Management
function initTheme() {
  const theme = localStorage.getItem('theme') || 'light';
  document.documentElement.setAttribute('data-theme', theme);
  updateThemeIcon(theme);
}

function toggleTheme() {
  const currentTheme = document.documentElement.getAttribute('data-theme');
  const newTheme = currentTheme === 'dark' ? 'light' : 'dark';
  document.documentElement.setAttribute('data-theme', newTheme);
  localStorage.setItem('theme', newTheme);
  updateThemeIcon(newTheme);
}

function updateThemeIcon(theme) {
  const themeToggle = document.getElementById('themeToggle');
  if (themeToggle) {
    themeToggle.textContent = theme === 'dark' ? '☀️' : '🌙';
  }
}

// Smooth Scrolling for Navigation
function initSmoothScroll() {
  document.querySelectorAll('a[href^="#"]').forEach(anchor => {
    anchor.addEventListener('click', function (e) {
      e.preventDefault();
      const target = document.querySelector(this.getAttribute('href'));
      if (target) {
        target.scrollIntoView({
          behavior: 'smooth',
          block: 'start'
        });
      }
    });
  });
}

// Copy Code to Clipboard
function copyToClipboard(button) {
  const codeBlock = button.closest('.code-section').querySelector('code');
  const text = codeBlock.textContent;
  
  navigator.clipboard.writeText(text).then(() => {
    const originalText = button.textContent;
    button.textContent = '✓ Copied!';
    button.style.background = 'var(--success-color)';
    button.style.color = 'white';
    
    setTimeout(() => {
      button.textContent = originalText;
      button.style.background = '';
      button.style.color = '';
    }, 2000);
  }).catch(err => {
    console.error('Failed to copy:', err);
    button.textContent = '✗ Failed';
    setTimeout(() => {
      button.textContent = '📋 Copy';
    }, 2000);
  });
}

// Add Copy Buttons to Code Blocks
function initCopyButtons() {
  document.querySelectorAll('.code-section').forEach(section => {
    const copyBtn = section.querySelector('.copy-btn');
    if (copyBtn) {
      copyBtn.addEventListener('click', () => copyToClipboard(copyBtn));
    }
  });
}

// Intersection Observer for Fade-in Animations
function initAnimations() {
  const observerOptions = {
    threshold: 0.1,
    rootMargin: '0px 0px -50px 0px'
  };
  
  const observer = new IntersectionObserver((entries) => {
    entries.forEach(entry => {
      if (entry.isIntersecting) {
        entry.target.classList.add('fade-in');
        observer.unobserve(entry.target);
      }
    });
  }, observerOptions);
  
  document.querySelectorAll('.feature-card, .flow-card, .api-endpoint').forEach(el => {
    observer.observe(el);
  });
}

// Scroll Progress Bar
function updateScrollProgress() {
  const progressBar = document.getElementById('scrollProgress');
  if (!progressBar) return;
  
  const windowHeight = window.innerHeight;
  const documentHeight = document.documentElement.scrollHeight - windowHeight;
  const scrolled = window.pageYOffset;
  const progress = (scrolled / documentHeight) * 100;
  
  requestAnimationFrame(() => {
    progressBar.style.width = `${Math.min(progress, 100)}%`;
  });
}

// Highlight Active Navigation
function highlightActiveNav() {
  const sections = document.querySelectorAll('section[id]');
  const navLinks = document.querySelectorAll('.nav-link');
  
  window.addEventListener('scroll', () => {
    let current = '';
    
    // Update progress bar
    updateScrollProgress();
    
    // Find current section
    sections.forEach(section => {
      const sectionTop = section.offsetTop;
      const sectionHeight = section.clientHeight;
      if (window.pageYOffset >= sectionTop - 200) {
        current = section.getAttribute('id');
      }
    });
    
    // Update active nav link
    navLinks.forEach(link => {
      link.classList.remove('active');
      if (link.getAttribute('href') === `#${current}`) {
        link.classList.add('active');
      }
    });
  });
  
  // Initial call
  updateScrollProgress();
}

// Search Functionality
function initSearch() {
  const searchInput = document.getElementById('searchInput');
  if (!searchInput) return;
  
  searchInput.addEventListener('input', (e) => {
    const searchTerm = e.target.value.toLowerCase();
    const sections = document.querySelectorAll('section');
    
    sections.forEach(section => {
      const text = section.textContent.toLowerCase();
      if (text.includes(searchTerm) || searchTerm === '') {
        section.style.display = '';
      } else {
        section.style.display = 'none';
      }
    });
  });
}

// Table of Contents Generation
function generateTOC() {
  const toc = document.getElementById('tableOfContents');
  if (!toc) return;
  
  const headings = document.querySelectorAll('h2[id], h3[id]');
  const tocList = document.createElement('ul');
  tocList.className = 'toc-list';
  
  headings.forEach(heading => {
    const li = document.createElement('li');
    const a = document.createElement('a');
    a.href = `#${heading.id}`;
    a.textContent = heading.textContent;
    a.className = heading.tagName === 'H3' ? 'toc-sub-item' : 'toc-item';
    li.appendChild(a);
    tocList.appendChild(li);
  });
  
  toc.appendChild(tocList);
}

// Mermaid Diagram Initialization
function initMermaid() {
  if (typeof mermaid !== 'undefined') {
    const theme = document.documentElement.getAttribute('data-theme') === 'dark' ? 'dark' : 'default';
    
    // Store original mermaid text before processing
    document.querySelectorAll('.mermaid').forEach((element) => {
      const originalText = element.textContent.trim();
      element.setAttribute('data-original-mermaid', originalText);
    });
    
    mermaid.initialize({
      startOnLoad: true,
      theme: theme,
      securityLevel: 'loose',
      flowchart: {
        useMaxWidth: true,
        htmlLabels: true,
        curve: 'basis'
      },
      sequence: {
        useMaxWidth: true,
        diagramMarginX: 50,
        diagramMarginY: 10
      },
      class: {
        useMaxWidth: true
      }
    });
    
    // Force render after initialization
    setTimeout(() => {
      try {
        mermaid.contentLoaded();
      } catch (err) {
        console.error('Mermaid initialization error:', err);
      }
    }, 100);
  }
}

// Re-render Mermaid on Theme Change
function reRenderMermaid() {
  if (typeof mermaid !== 'undefined') {
    const theme = document.documentElement.getAttribute('data-theme') === 'dark' ? 'dark' : 'default';
    
    // Store original diagram definitions before any processing
    const diagramData = [];
    document.querySelectorAll('.mermaid').forEach((element) => {
      // Get the original text from the element
      let originalText = element.getAttribute('data-original-mermaid');
      if (!originalText) {
        // First time - store the original
        originalText = element.textContent.trim();
        element.setAttribute('data-original-mermaid', originalText);
      }
      diagramData.push({ element, text: originalText });
    });
    
    // Reinitialize mermaid with new theme
    mermaid.initialize({ 
      startOnLoad: false,
      theme: theme,
      securityLevel: 'loose',
      flowchart: { useMaxWidth: true, htmlLabels: true, curve: 'basis' },
      sequence: { useMaxWidth: true, diagramMarginX: 50, diagramMarginY: 10 },
      class: { useMaxWidth: true }
    });
    
    // Re-render each diagram
    diagramData.forEach(({ element, text }, index) => {
      element.removeAttribute('data-processed');
      element.innerHTML = text;
      
      // Generate unique ID for this diagram
      const id = `mermaid-diagram-${index}-${Date.now()}`;
      
      try {
        mermaid.render(id, text).then(({ svg }) => {
          element.innerHTML = svg;
        }).catch(err => {
          console.error('Mermaid render error:', err);
          element.innerHTML = `<div style="color: var(--danger-color); padding: 1rem;">Error rendering diagram</div>`;
        });
      } catch (err) {
        console.error('Mermaid render error:', err);
        element.innerHTML = `<div style="color: var(--danger-color); padding: 1rem;">Error rendering diagram</div>`;
      }
    });
  }
}

// Enhanced Theme Toggle with Mermaid
function toggleThemeWithMermaid() {
  toggleTheme();
  setTimeout(reRenderMermaid, 100);
}

// API Request Demo
async function testAPIEndpoint(endpoint, method = 'GET', body = null) {
  const demoSection = document.getElementById('apiDemo');
  if (!demoSection) return;
  
  const resultDiv = demoSection.querySelector('.demo-result');
  resultDiv.innerHTML = '<div class="loading"></div> Testing endpoint...';
  
  try {
    const options = {
      method,
      headers: { 'Content-Type': 'application/json' }
    };
    
    if (body) {
      options.body = JSON.stringify(body);
    }
    
    const response = await fetch(`http://localhost:3000${endpoint}`, options);
    const data = await response.json();
    
    resultDiv.innerHTML = `
      <div class="success-result">
        <strong>Status:</strong> ${response.status} ${response.statusText}<br>
        <strong>Response:</strong>
        <pre><code>${JSON.stringify(data, null, 2)}</code></pre>
      </div>
    `;
  } catch (error) {
    resultDiv.innerHTML = `
      <div class="error-result">
        <strong>Error:</strong> ${error.message}<br>
        <em>Note: Make sure the server is running on localhost:3000</em>
      </div>
    `;
  }
}

// Stats Counter Animation
function animateStats() {
  const statElements = document.querySelectorAll('.stat-number');
  
  statElements.forEach(stat => {
    const target = parseInt(stat.getAttribute('data-target'));
    const duration = 2000;
    const increment = target / (duration / 16);
    let current = 0;
    
    const updateCounter = () => {
      current += increment;
      if (current < target) {
        stat.textContent = Math.floor(current);
        requestAnimationFrame(updateCounter);
      } else {
        stat.textContent = target;
      }
    };
    
    const observer = new IntersectionObserver((entries) => {
      if (entries[0].isIntersecting) {
        updateCounter();
        observer.disconnect();
      }
    });
    
    observer.observe(stat);
  });
}

// Mobile Menu Toggle
function initMobileMenu() {
  const menuToggle = document.getElementById('menuToggle');
  const navMenu = document.getElementById('navMenu');
  
  if (menuToggle && navMenu) {
    menuToggle.addEventListener('click', (e) => {
      e.stopPropagation();
      navMenu.classList.toggle('active');
      menuToggle.classList.toggle('active');
    });
    
    // Close menu when clicking on a nav link
    const navLinks = navMenu.querySelectorAll('.nav-link');
    navLinks.forEach(link => {
      link.addEventListener('click', () => {
        navMenu.classList.remove('active');
        menuToggle.classList.remove('active');
      });
    });
    
    // Close menu when clicking outside
    document.addEventListener('click', (e) => {
      if (!navMenu.contains(e.target) && !menuToggle.contains(e.target)) {
        navMenu.classList.remove('active');
        menuToggle.classList.remove('active');
      }
    });
    
    // Close menu on escape key
    document.addEventListener('keydown', (e) => {
      if (e.key === 'Escape') {
        navMenu.classList.remove('active');
        menuToggle.classList.remove('active');
      }
    });
  }
}

// Back to Top Button
function initBackToTop() {
  const backToTop = document.createElement('button');
  backToTop.innerHTML = '↑';
  backToTop.className = 'back-to-top';
  backToTop.style.cssText = `
    position: fixed;
    bottom: 2rem;
    right: 2rem;
    width: 50px;
    height: 50px;
    border-radius: 50%;
    background: var(--primary-color);
    color: white;
    border: none;
    cursor: pointer;
    font-size: 1.5rem;
    display: none;
    box-shadow: var(--shadow-md);
    transition: var(--transition);
    z-index: 999;
  `;
  
  document.body.appendChild(backToTop);
  
  window.addEventListener('scroll', () => {
    if (window.pageYOffset > 300) {
      backToTop.style.display = 'block';
    } else {
      backToTop.style.display = 'none';
    }
  });
  
  backToTop.addEventListener('click', () => {
    window.scrollTo({ top: 0, behavior: 'smooth' });
  });
  
  backToTop.addEventListener('mouseenter', () => {
    backToTop.style.transform = 'scale(1.1)';
  });
  
  backToTop.addEventListener('mouseleave', () => {
    backToTop.style.transform = 'scale(1)';
  });
}

// Code Syntax Highlighting (basic)
function initSyntaxHighlight() {
  document.querySelectorAll('pre code').forEach(block => {
    // Basic syntax highlighting for common patterns
    let html = block.innerHTML;
    
    // Highlight strings
    html = html.replace(/"([^"]*)"/g, '<span style="color: var(--success-color);">"$1"</span>');
    
    // Highlight numbers
    html = html.replace(/\b(\d+)\b/g, '<span style="color: var(--accent-color);">$1</span>');
    
    // Highlight comments
    html = html.replace(/(#.*$)/gm, '<span style="color: var(--text-secondary); font-style: italic;">$1</span>');
    
    block.innerHTML = html;
  });
}

// Print Page Setup
function setupPrint() {
  window.addEventListener('beforeprint', () => {
    document.body.classList.add('printing');
  });
  
  window.addEventListener('afterprint', () => {
    document.body.classList.remove('printing');
  });
}

// Initialize All Features
function init() {
  console.log('🔐 Passwordless Auth Wiki - Initializing...');
  
  initTheme();
  initSmoothScroll();
  initCopyButtons();
  initAnimations();
  highlightActiveNav();
  initSearch();
  generateTOC();
  initMermaid();
  animateStats();
  initMobileMenu();
  initBackToTop();
  setupPrint();
  
  console.log('✅ Wiki initialized successfully!');
}

// Run on DOM Content Loaded
if (document.readyState === 'loading') {
  document.addEventListener('DOMContentLoaded', init);
} else {
  init();
}

// Export functions for global access
window.passwordlessAuth = {
  toggleTheme: toggleThemeWithMermaid,
  testAPI: testAPIEndpoint,
  copyCode: copyToClipboard
};
