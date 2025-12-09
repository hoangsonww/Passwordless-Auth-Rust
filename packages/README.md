# Packages Directory

This directory contains the assets for the Passwordless Auth wiki page.

## Files

### `styles.css` (12.6 KB)
Professional CSS styling with:
- CSS variables for easy theming
- Light/dark mode support
- Responsive design (mobile-first)
- Modern animations and transitions
- Clean, readable typography
- Accessibility features

### `script.js` (10.5 KB)
Interactive JavaScript features:
- Theme toggle with localStorage persistence
- Copy-to-clipboard for code blocks
- Smooth scrolling navigation
- Intersection Observer animations
- Back-to-top button
- Mermaid diagram initialization
- Mobile menu support
- Syntax highlighting

## Usage

These files are automatically loaded by `index.html` in the project root.

```html
<link rel="stylesheet" href="packages/styles.css">
<script src="packages/script.js"></script>
```

## Customization

### Changing Theme Colors

Edit `styles.css` and modify the CSS variables:

```css
:root {
  --primary-color: #0066cc;
  --accent-color: #ff6b35;
  /* ... more variables ... */
}
```

### Adding New Features

Edit `script.js` and add your functions to the initialization:

```javascript
function init() {
  // ... existing initialization ...
  yourNewFeature();
}
```

## Dependencies

- **External:** Mermaid.js (loaded via CDN for diagrams)
- **Internal:** No build process required, vanilla JavaScript

## Browser Compatibility

- Chrome/Edge 90+
- Firefox 88+
- Safari 14+
- Mobile browsers (iOS Safari, Chrome Mobile)

## Performance

- Total size: ~23 KB (uncompressed)
- No external dependencies except Mermaid
- Lazy-loaded animations
- Optimized DOM queries
