# BioMiner Indexd Documentation

This directory contains the GitHub Pages documentation website for BioMiner Indexd.

## Overview

The documentation website provides comprehensive information about BioMiner Indexd, including:

- **Home Page**: Project introduction and overview
- **Features**: Detailed feature descriptions and technical architecture
- **Copilot**: AI assistant functionality and usage guide
- **Datasets**: Dataset building guide and examples
- **GitHub**: Repository links and contribution guidelines

## File Structure

```
docs/
├── index.html              # Home page
├── features.html           # Features page
├── copilot.html            # AI Copilot page
├── datasets.html           # Datasets page
├── github.html             # GitHub page
├── styles.css              # Main stylesheet
├── script.js               # JavaScript functionality
├── _config.yml             # GitHub Pages configuration
├── build_dataset_en.md     # Dataset building documentation
├── build_dataset_cn.md     # Chinese dataset building documentation
└── README.md               # This file
```

## Features

### Modern Design
- Responsive design that works on all devices
- Modern gradient backgrounds and card-based layouts
- Smooth animations and hover effects
- Professional typography and spacing

### Interactive Elements
- Animated elements that appear on scroll
- Interactive dataset cards with hover effects
- Simulated AI chat interface
- Code copy buttons
- Smooth scrolling navigation

### Content Organization
- Clear navigation structure
- Comprehensive feature documentation
- Step-by-step guides
- Best practices and examples
- Community guidelines

## Deployment

### GitHub Pages Setup

1. **Enable GitHub Pages**:
   - Go to your repository settings
   - Navigate to "Pages" section
   - Select "Deploy from a branch"
   - Choose the `main` branch and `/docs` folder
   - Click "Save"

2. **Custom Domain (Optional)**:
   - Add your custom domain in the Pages settings
   - Update the `url` in `_config.yml` if needed

### Local Development

1. **Install Jekyll** (if using Jekyll features):
   ```bash
   gem install jekyll bundler
   bundle install
   ```

2. **Run locally**:
   ```bash
   jekyll serve
   ```

3. **View site**: Open `http://localhost:4000` in your browser

### Static HTML Deployment

Since this is a static HTML site, you can also deploy it to any static hosting service:

- **Netlify**: Drag and drop the `docs` folder
- **Vercel**: Connect your GitHub repository
- **AWS S3**: Upload files to an S3 bucket
- **Any web server**: Upload files to your web server

## Customization

### Colors and Theme

The site uses CSS custom properties for easy theming. Edit `styles.css`:

```css
:root {
    --primary-color: #007bff;      /* Main brand color */
    --secondary-color: #6c757d;    /* Secondary color */
    --success-color: #28a745;      /* Success states */
    --info-color: #17a2b8;         /* Info states */
    --warning-color: #ffc107;      /* Warning states */
    --danger-color: #dc3545;       /* Error states */
}
```

### Content Updates

1. **Edit HTML files** directly to update content
2. **Update navigation** in all HTML files when adding new pages
3. **Modify `_config.yml`** for site-wide settings
4. **Update JavaScript** in `script.js` for new interactions

### Adding New Pages

1. Create a new HTML file (e.g., `new-page.html`)
2. Copy the structure from an existing page
3. Update the navigation in all HTML files
4. Add any new styles to `styles.css`
5. Add any new JavaScript to `script.js`

## Content Guidelines

### Writing Style
- Use clear, concise language
- Write for both technical and non-technical audiences
- Include code examples where appropriate
- Use consistent terminology

### Images and Media
- Optimize images for web (compress, use appropriate formats)
- Include alt text for accessibility
- Use descriptive filenames
- Consider lazy loading for performance

### Links
- Use relative links for internal pages
- Use absolute URLs for external resources
- Test all links before deployment
- Include appropriate target attributes for external links

## Performance Optimization

### Current Optimizations
- Minified CSS and JavaScript (when deployed)
- Optimized images
- Lazy loading for images
- Debounced scroll events
- Efficient DOM queries

### Additional Optimizations
- Enable gzip compression on your server
- Use a CDN for external resources
- Implement browser caching
- Consider using WebP images with fallbacks

## SEO and Analytics

### SEO Features
- Proper meta tags in all pages
- Structured data markup
- Semantic HTML structure
- Optimized page titles and descriptions

### Analytics Setup
1. **Google Analytics**: Add your tracking ID to `_config.yml`
2. **Search Console**: Submit your sitemap
3. **Social Media**: Update Open Graph tags

## Maintenance

### Regular Tasks
- Update content as the project evolves
- Check and fix broken links
- Update dependencies and libraries
- Monitor performance metrics
- Review and update documentation

### Version Control
- Commit changes regularly
- Use descriptive commit messages
- Tag releases for major updates
- Keep a changelog of significant changes

## Troubleshooting

### Common Issues

1. **Styles not loading**:
   - Check file paths
   - Verify CSS file is in the correct location
   - Clear browser cache

2. **JavaScript not working**:
   - Check browser console for errors
   - Verify script file is loaded
   - Check for JavaScript conflicts

3. **Images not displaying**:
   - Verify image file paths
   - Check file permissions
   - Ensure images are in the correct format

4. **Navigation issues**:
   - Verify all HTML files have consistent navigation
   - Check for typos in file names
   - Test all links

### Getting Help

- Check the browser console for error messages
- Validate HTML and CSS using online validators
- Test in different browsers and devices
- Review GitHub Pages documentation for deployment issues

## Contributing

To contribute to the documentation:

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Test locally
5. Submit a pull request

### Documentation Standards

- Follow the existing style and structure
- Include appropriate alt text for images
- Test all links and functionality
- Update navigation when adding new pages
- Keep content up to date with the project

## License

This documentation is part of the BioMiner Indexd project and is distributed under the same license as the main project (GNU Affero General Public License v3.0).

---

For more information about BioMiner Indexd, visit the main project repository: [https://github.com/yjcyxky/biominer-indexd](https://github.com/yjcyxky/biominer-indexd) 