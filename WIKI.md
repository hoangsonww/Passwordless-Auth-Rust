# Passwordless Auth - Wiki Page

## Overview

This directory contains a professional, interactive wiki page for the Passwordless Auth project.

## Files

- **`index.html`** - Main wiki page with comprehensive documentation
- **`packages/styles.css`** - Professional styling with light/dark theme support
- **`packages/script.js`** - Interactive features and animations

## Features

### 🎨 Design
- Modern, responsive design
- Light/dark theme toggle
- Smooth animations and transitions
- Professional color scheme
- Mobile-friendly layout

### 📊 Content
- Complete project overview
- Architecture diagrams using Mermaid
- Full API documentation
- Quick start guide
- Security considerations
- Troubleshooting guide
- Glossary

### ⚡ Interactive Features
- Theme switcher (persists to localStorage)
- Copy-to-clipboard for code blocks
- Smooth scrolling navigation
- Animated sections on scroll
- Back-to-top button
- Responsive navigation

## Serving the Wiki

### Option 1: Simple HTTP Server (Python)

```bash
# Navigate to project root
cd /Users/davidnguyen/WebstormProjects/Passwordless-Auth-Rust

# Python 3
python3 -m http.server 8080

# Python 2
python -m SimpleHTTPServer 8080

# Then open: http://localhost:8080
```

### Option 2: Node.js Server

```bash
# Install http-server globally
npm install -g http-server

# Serve the directory
http-server -p 8080

# Then open: http://localhost:8080
```

### Option 3: Using Rust (miniserve)

```bash
# Install miniserve
cargo install miniserve

# Serve the directory
miniserve . -p 8080

# Then open: http://localhost:8080
```

### Option 4: Nginx (Production)

```nginx
server {
    listen 80;
    server_name wiki.yourcompany.com;
    
    root /path/to/passwordless-auth;
    index index.html;
    
    location / {
        try_files $uri $uri/ =404;
    }
    
    # Enable compression
    gzip on;
    gzip_types text/css application/javascript application/json image/svg+xml;
}
```

### Option 5: Docker

```bash
# Create a simple Dockerfile for the wiki
cat > Dockerfile.wiki <<EOF
FROM nginx:alpine
COPY index.html /usr/share/nginx/html/
COPY packages /usr/share/nginx/html/packages
EXPOSE 80
EOF

# Build and run
docker build -f Dockerfile.wiki -t passwordless-wiki .
docker run -p 8080:80 passwordless-wiki
```

## Customization

### Changing Colors

Edit `packages/styles.css` and modify the CSS variables in the `:root` section:

```css
:root {
  --primary-color: #0066cc;  /* Change primary color */
  --accent-color: #ff6b35;   /* Change accent color */
  /* ... other variables ... */
}
```

### Adding Content

Edit `index.html` and add new sections following the existing structure:

```html
<section id="your-section" class="section">
    <h2 class="section-title">Your Section Title</h2>
    <p class="section-subtitle">Your subtitle</p>
    
    <!-- Your content here -->
</section>
```

### Modifying Diagrams

The Mermaid diagrams can be edited directly in the `index.html` file within `<div class="mermaid">` tags.

## Browser Support

- ✅ Chrome/Edge (latest)
- ✅ Firefox (latest)
- ✅ Safari (latest)
- ✅ Mobile browsers

## Performance

- Minimal dependencies (only Mermaid CDN)
- Optimized CSS with CSS variables
- Efficient JavaScript with modern features
- Fast page load times

## Accessibility

- Semantic HTML structure
- ARIA labels where appropriate
- Keyboard navigation support
- Color contrast compliance
- Responsive font sizes

## License

Same as the main project: MIT License

---

**Note:** This wiki page is designed to be served as a static site and does not require a backend server. All interactions happen client-side.
