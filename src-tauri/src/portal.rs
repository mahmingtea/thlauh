use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct FileItem {
    pub name: String,
    #[serde(rename = "isDirectory")]
    pub is_directory: bool,
    pub size: u64,
}

/// Generate the web portal HTML page that remote browsers see when accessing the shared directory.
/// This is a direct port of the TypeScript `generateDirectoryHtml()` from portal.ts.
pub fn generate_directory_html(
    _dir_path: &str,
    req_path: &str,
    files: &[FileItem],
    theme: &str,
) -> String {
    // Normalize reqPath
    let clean_req_path = req_path.trim_matches('/');
    let segments: Vec<&str> = clean_req_path.split('/').filter(|s| !s.is_empty()).collect();

    let mut breadcrumb_html = r#"<a href="/" class="root-link">Root</a>"#.to_string();
    let mut accumulated = String::new();
    for (i, seg) in segments.iter().enumerate() {
        accumulated.push('/');
        accumulated.push_str(seg);
        breadcrumb_html.push_str(&format!(
            r#" <span class="breadcrumbs-separator">/</span> <a href="{}">{}</a>"#,
            accumulated, seg
        ));
        let _ = i; // used in loop
    }

    if !segments.is_empty() {
        let parent_path = if segments.len() > 1 {
            format!("/{}", segments[..segments.len() - 1].join("/"))
        } else {
            "/".to_string()
        };
        breadcrumb_html = format!(
            r#"<a href="{parent}" class="breadcrumbs-back-btn" title="Go up one folder">
                <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><line x1="12" y1="19" x2="12" y2="5"></line><polyline points="5 12 12 5 19 12"></polyline></svg>
              </a>
              <span class="breadcrumbs-separator" style="margin-right: 0.5rem; margin-left: 0.25rem;">|</span>
              {bc}"#,
            parent = parent_path,
            bc = breadcrumb_html
        );
    }

    let files_json = serde_json::to_string(files).unwrap_or_else(|_| "[]".to_string());
    let req_path_json = serde_json::to_string(req_path).unwrap_or_else(|_| r#""""#.to_string());

    format!(
        r##"<!DOCTYPE html>
<html lang="en" class="{theme}">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0, maximum-scale=1.0, user-scalable=no">
  <title>Thlauh Share</title>
  
  <style>
    :root {{
      --background: oklch(1 0 0);
      --foreground: oklch(0.145 0 0);
      --card: oklch(1 0 0);
      --card-foreground: oklch(0.145 0 0);
      --popover: oklch(1 0 0);
      --popover-foreground: oklch(0.145 0 0);
      --primary: oklch(0.205 0 0);
      --primary-foreground: oklch(0.985 0 0);
      --secondary: oklch(0.97 0 0);
      --secondary-foreground: oklch(0.205 0 0);
      --muted: oklch(0.97 0 0);
      --muted-foreground: oklch(0.556 0 0);
      --accent: oklch(0.97 0 0);
      --accent-foreground: oklch(0.205 0 0);
      --destructive: oklch(0.577 0.245 27.325);
      --border: oklch(0.922 0 0);
      --input: oklch(0.922 0 0);
      --ring: oklch(0.708 0 0);
    }}

    .dark {{
      --background: oklch(0.145 0 0);
      --foreground: oklch(0.985 0 0);
      --card: oklch(0.205 0 0);
      --card-foreground: oklch(0.985 0 0);
      --popover: oklch(0.205 0 0);
      --popover-foreground: oklch(0.985 0 0);
      --primary: oklch(0.922 0 0);
      --primary-foreground: oklch(0.205 0 0);
      --secondary: oklch(0.269 0 0);
      --secondary-foreground: oklch(0.985 0 0);
      --muted: oklch(0.269 0 0);
      --muted-foreground: oklch(0.708 0 0);
      --accent: oklch(0.269 0 0);
      --accent-foreground: oklch(0.985 0 0);
      --destructive: oklch(0.704 0.191 22.216);
      --border: oklch(1 0 0 / 10%);
      --input: oklch(1 0 0 / 10%);
      --ring: oklch(0.708 0 0);
    }}

    html, body {{
      background-color: var(--background);
      color: var(--foreground);
      font-family: system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
      margin: 0;
      padding: 0;
      min-height: 100vh;
      display: flex;
      flex-direction: column;
      -webkit-font-smoothing: antialiased;
      overflow-x: hidden;
      width: 100%;
    }}

    .app-container {{
      display: flex;
      flex-direction: column;
      flex-grow: 1;
      min-height: 100vh;
    }}

    header {{
      display: flex;
      align-items: center;
      justify-content: space-between;
      padding: 0.75rem 1.5rem;
      border-bottom: 1px solid var(--border);
      background-color: color-mix(in srgb, var(--card) 40%, transparent);
      backdrop-filter: blur(12px);
      -webkit-backdrop-filter: blur(12px);
      z-index: 10;
      flex-shrink: 0;
    }}

    .header-logo-container {{
      display: flex;
      align-items: center;
      gap: 0.75rem;
    }}

    .icon-box {{
      padding: 0.5rem;
      border-radius: 0.5rem;
      background-color: color-mix(in srgb, var(--primary) 10%, transparent);
      color: var(--primary);
      border: 1px solid color-mix(in srgb, var(--primary) 20%, transparent);
      display: flex;
      align-items: center;
      justify-content: center;
    }}

    h1 {{
      font-size: 1.125rem;
      font-weight: 700;
      letter-spacing: -0.025em;
      margin: 0;
      line-height: 1.25;
    }}

    .sub-desc {{
      font-size: 0.75rem;
      color: var(--muted-foreground);
      margin: 0;
    }}

    main {{
      flex-grow: 1;
      padding: 1.5rem;
      max-width: 72rem;
      margin: 0 auto;
      width: 100%;
      box-sizing: border-box;
      display: grid;
      grid-template-columns: 1fr;
      gap: 1.5rem;
      z-index: 10;
      align-items: start;
    }}

    .upload-section {{ width: 100%; }}
    .files-section {{ width: 100%; }}

    @media (min-width: 768px) {{
      main {{
        grid-template-columns: repeat(12, minmax(0, 1fr));
      }}
      .upload-section {{
        grid-column: span 4 / span 4;
      }}
      .files-section {{
        grid-column: span 8 / span 8;
      }}
    }}

    .card {{
      border: 0;
      background-color: transparent;
      padding: 0;
      display: flex;
      flex-direction: column;
    }}

    @media (min-width: 768px) {{
      .card {{
        border: 1px solid var(--border);
        background-color: color-mix(in srgb, var(--card) 40%, transparent);
        border-radius: 0.75rem;
        padding: 1.25rem;
        box-shadow: 0 1px 2px 0 rgb(0 0 0 / 0.05);
      }}
    }}

    .card h2 {{
      font-size: 1rem;
      font-weight: 600;
      margin: 0 0 0.5rem 0;
      display: flex;
      align-items: center;
      gap: 0.5rem;
    }}

    .card-subtitle {{
      font-size: 0.75rem;
      color: var(--muted-foreground);
      margin: 0 0 1rem 0;
    }}

    .dropzone {{
      border: 2px dashed var(--border);
      border-radius: 0.5rem;
      padding: 1.5rem;
      text-align: center;
      cursor: pointer;
      transition: all 0.2s ease-in-out;
      background-color: color-mix(in srgb, var(--background) 20%, transparent);
      display: flex;
      flex-direction: column;
      align-items: center;
      justify-content: center;
    }}

    .dropzone:hover {{
      border-color: color-mix(in srgb, var(--primary) 50%, transparent);
      background-color: color-mix(in srgb, var(--primary) 2%, transparent);
    }}

    .dropzone-icon {{
      color: var(--muted-foreground);
      margin-bottom: 0.5rem;
      display: flex;
      align-items: center;
      justify-content: center;
    }}

    .dropzone-text {{ font-size: 0.875rem; font-weight: 500; margin: 0; }}
    .dropzone-subtext {{ font-size: 0.75rem; color: var(--muted-foreground); margin: 0.25rem 0 0 0; }}

    .progress-container {{
      margin-top: 1rem;
      display: flex;
      flex-direction: column;
      gap: 0.5rem;
    }}

    .progress-info {{
      display: flex;
      justify-content: space-between;
      font-size: 0.75rem;
      font-weight: 600;
    }}

    .progress-filename {{
      white-space: nowrap;
      overflow: hidden;
      text-overflow: ellipsis;
      max-width: 150px;
    }}

    .progress-track {{
      width: 100%;
      background-color: var(--secondary);
      border-radius: 9999px;
      height: 0.375rem;
      overflow: hidden;
    }}

    .progress-bar {{
      background-color: var(--primary);
      height: 100%;
      border-radius: 9999px;
      transition: width 0.1s ease;
      width: 0%;
    }}

    .explorer-header {{
      padding: 0 0 1rem 0;
      border-bottom: 1px solid color-mix(in srgb, var(--border) 60%, transparent);
      margin-bottom: 1rem;
    }}

    .breadcrumbs {{
      display: flex;
      align-items: center;
      font-size: 0.75rem;
      color: var(--muted-foreground);
      font-weight: 600;
      overflow-x: auto;
      white-space: nowrap;
      padding-bottom: 0.25rem;
      margin-bottom: 0.5rem;
    }}

    .breadcrumbs::-webkit-scrollbar {{ display: none; }}

    .breadcrumbs a {{
      color: var(--primary);
      text-decoration: none;
      transition: color 0.15s ease;
    }}

    .breadcrumbs a:hover {{
      color: color-mix(in srgb, var(--primary) 80%, transparent);
    }}

    .breadcrumbs-separator {{
      color: color-mix(in srgb, var(--muted-foreground) 45%, transparent);
      margin: 0 0.375rem;
    }}

    .breadcrumbs-back-btn {{
      display: flex;
      align-items: center;
      justify-content: center;
      padding: 0.25rem;
      border-radius: 0.375rem;
      color: var(--muted-foreground);
      text-decoration: none;
      transition: all 0.15s ease;
    }}

    .breadcrumbs-back-btn:hover {{
      background-color: var(--muted);
      color: var(--foreground);
    }}

    .explorer-toolbar {{
      display: flex;
      align-items: center;
      justify-content: space-between;
      gap: 1rem;
    }}

    .search-input {{
      background-color: color-mix(in srgb, var(--background) 60%, transparent);
      border: 1px solid var(--input);
      color: var(--foreground);
      font-size: 0.75rem;
      padding: 0.375rem 0.75rem;
      border-radius: 0.375rem;
      width: 9rem;
      transition: all 0.15s ease;
      box-sizing: border-box;
    }}

    @media (min-width: 640px) {{
      .search-input {{ width: 12rem; }}
    }}

    .search-input:focus {{
      outline: none;
      border-color: color-mix(in srgb, var(--primary) 50%, transparent);
    }}

    .mobile-cards-grid {{
      display: grid;
      grid-template-columns: 1fr;
      gap: 0.75rem;
      padding: 1rem 0;
    }}

    @media (min-width: 768px) {{
      .mobile-cards-grid {{ display: none; }}
    }}

    .file-card {{
      background-color: color-mix(in srgb, var(--card) 50%, transparent);
      border: 1px solid color-mix(in srgb, var(--border) 60%, transparent);
      border-radius: 0.5rem;
      padding: 0.875rem;
      display: flex;
      flex-direction: column;
      gap: 0.75rem;
      transition: border-color 0.2s ease;
    }}

    .file-card:hover {{
      border-color: color-mix(in srgb, var(--primary) 45%, transparent);
    }}

    .file-card-info {{
      display: flex;
      align-items: flex-start;
      gap: 0.75rem;
      min-width: 0;
    }}

    .preview-box {{
      width: 2.5rem;
      height: 2.5rem;
      display: flex;
      align-items: center;
      justify-content: center;
      flex-shrink: 0;
    }}

    .preview-image, .preview-video {{
      width: 2.5rem;
      height: 2.5rem;
      object-fit: cover;
      border-radius: 0.375rem;
      border: 1px solid var(--border);
    }}

    .file-card-details {{
      min-width: 0;
      flex-grow: 1;
    }}

    .file-name-title {{
      font-size: 0.875rem;
      font-weight: 500;
      margin: 0;
      white-space: nowrap;
      overflow: hidden;
      text-overflow: ellipsis;
      color: color-mix(in srgb, var(--foreground) 90%, transparent);
    }}

    .file-meta-row {{
      display: flex;
      align-items: center;
      gap: 0.5rem;
      margin-top: 0.25rem;
    }}

    .badge {{
      font-size: 9px;
      background-color: var(--background);
      border: 1px solid var(--input);
      color: var(--muted-foreground);
      padding: 0.125rem 0.375rem;
      border-radius: 0.25rem;
      text-transform: uppercase;
      font-weight: 600;
    }}

    .file-size-label {{
      font-size: 10px;
      color: color-mix(in srgb, var(--muted-foreground) 60%, transparent);
      font-family: monospace;
    }}

    .action-btn {{
      width: 100%;
      text-align: center;
      padding: 0.5rem 0;
      border-radius: 0.375rem;
      font-size: 0.75rem;
      font-weight: 600;
      text-decoration: none;
      transition: background-color 0.15s ease;
      box-sizing: border-box;
    }}

    .btn-folder {{
      background-color: color-mix(in srgb, var(--primary) 10%, transparent);
      color: var(--primary);
      border: 1px solid color-mix(in srgb, var(--primary) 20%, transparent);
    }}
    .btn-folder:hover {{
      background-color: color-mix(in srgb, var(--primary) 20%, transparent);
    }}

    .btn-file {{
      background-color: oklch(0.866 0.127 151.78 / 10%);
      color: oklch(0.666 0.127 151.78);
      border: 1px solid oklch(0.666 0.127 151.78 / 20%);
    }}
    .btn-file:hover {{
      background-color: oklch(0.866 0.127 151.78 / 20%);
    }}

    .dark .btn-file {{
      background-color: oklch(0.766 0.127 151.78 / 10%);
      color: oklch(0.766 0.127 151.78);
      border: 1px solid oklch(0.766 0.127 151.78 / 20%);
    }}
    .dark .btn-file:hover {{
      background-color: oklch(0.766 0.127 151.78 / 20%);
    }}

    .desktop-table-container {{
      display: none;
      overflow-x: auto;
    }}

    @media (min-width: 768px) {{
      .desktop-table-container {{ display: block; }}
    }}

    table {{
      width: 100%;
      border-collapse: collapse;
      text-align: left;
      font-size: 0.875rem;
    }}

    thead {{
      background-color: color-mix(in srgb, var(--background) 40%, transparent);
      color: var(--muted-foreground);
      font-size: 0.75rem;
      font-weight: 600;
      border-bottom: 1px solid color-mix(in srgb, var(--border) 60%, transparent);
      text-transform: uppercase;
    }}

    th {{ padding: 0.5rem 1rem; }}

    tr.file-row {{
      border-bottom: 1px solid color-mix(in srgb, var(--border) 40%, transparent);
      transition: background-color 0.15s ease;
    }}

    tr.file-row:hover {{
      background-color: color-mix(in srgb, var(--muted) 20%, transparent);
    }}

    td {{
      padding: 0.75rem 1rem;
      vertical-align: middle;
    }}

    .td-name-cell {{
      display: flex;
      align-items: center;
      gap: 0.625rem;
      min-width: 0;
    }}

    .td-name-cell a {{
      color: var(--foreground);
      text-decoration: none;
      font-weight: 500;
      transition: color 0.15s ease;
      white-space: nowrap;
      overflow: hidden;
      text-overflow: ellipsis;
      max-width: 20rem;
    }}

    @media (min-width: 1024px) {{
      .td-name-cell a {{ max-width: 28rem; }}
    }}

    .td-name-cell a:hover {{ color: var(--primary); }}

    .td-type {{ color: var(--muted-foreground); font-size: 0.75rem; }}
    .td-size {{ color: var(--muted-foreground); font-size: 0.75rem; font-family: monospace; }}
    .td-action {{ text-align: right; }}

    .td-action-link {{
      font-size: 0.75rem;
      font-weight: 600;
      text-decoration: none;
      transition: color 0.15s ease;
    }}

    .td-action-folder {{ color: var(--primary); }}
    .td-action-folder:hover {{ color: color-mix(in srgb, var(--primary) 80%, transparent); }}

    .td-action-file {{ color: oklch(0.666 0.127 151.78); }}
    .td-action-file:hover {{ color: oklch(0.566 0.127 151.78); }}

    .dark .td-action-file {{ color: oklch(0.766 0.127 151.78); }}
    .dark .td-action-file:hover {{ color: oklch(0.866 0.127 151.78); }}

    .hidden {{ display: none !important; }}
    
    .scrollbar-none::-webkit-scrollbar {{ display: none; }}
    .scrollbar-none {{
      -ms-overflow-style: none;
      scrollbar-width: none;
    }}
  </style>
</head>
<body>
  
  <div class="app-container">
    <header>
      <div class="header-logo-container">
        <div class="icon-box" style="padding: 0.125rem;">
          <img src="/logo.png" style="width: 1.5rem; height: 1.5rem; object-fit: contain;" alt="Thlauh!" />
        </div>
        <div>
          <h1>Thlauh! Share</h1>
          <p class="sub-desc">File in share-na awlsam</p>
        </div>
      </div>
    </header>

    <main>
      <div class="upload-section">
        <div class="card">
          <h2>
            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" style="color: var(--primary);"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path><polyline points="17 8 12 3 7 8"></polyline><line x1="12" y1="3" x2="12" y2="15"></line></svg>
            Upload Files
          </h2>
          <p class="card-subtitle">Upload files directly into this shared directory.</p>

          <div id="dropzone" class="dropzone">
            <input type="file" id="file-input" class="hidden" multiple />
            <div class="dropzone-icon">
              <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><rect width="18" height="18" x="3" y="3" rx="2" ry="2"></rect><polyline points="11 3 11 11 14 8 17 11 17 3"></polyline></svg>
            </div>
            <p class="dropzone-text">Click to select files</p>
            <p class="dropzone-subtext">or drag &amp; drop them here</p>
          </div>

          <div id="progress-container" class="progress-container hidden">
            <div class="progress-info">
              <span id="progress-filename" class="progress-filename">File</span>
              <span id="progress-percentage">0%</span>
            </div>
            <div class="progress-track">
              <div id="progress-bar" class="progress-bar"></div>
            </div>
          </div>
        </div>
      </div>

      <div class="files-section">
        <div class="card">
          <div class="explorer-header">
            <div class="breadcrumbs">
              {breadcrumb}
            </div>
            
            <div class="explorer-toolbar">
              <h2 id="files-count-badge">
                <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" style="color: var(--primary);"><path d="M20 20a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-7.9a2 2 0 0 1-1.69-.9L9.6 3.9A2 2 0 0 0 7.93 3H4a2 2 0 0 0-2 2v13a2 2 0 0 0 2 2Z"></path></svg>
                Files ({file_count})
              </h2>
              
              <input 
                type="text"
                id="search-input"
                placeholder="Search files..."
                class="search-input"
              />
            </div>
          </div>

          <div id="mobile-cards-container" class="mobile-cards-grid"></div>

          <div class="desktop-table-container">
            <table>
              <thead>
                <tr>
                  <th style="padding-left: 1rem;">Name</th>
                  <th>Type</th>
                  <th>Size</th>
                  <th style="text-align: right; padding-right: 1rem;">Action</th>
                </tr>
              </thead>
              <tbody id="desktop-table-body"></tbody>
            </table>
          </div>
        </div>
      </div>
    </main>
  </div>

  <script>
    window.__INITIAL_DATA__ = {{
      reqPath: {req_path_json},
      files: {files_json}
    }};
  </script>

  <script>
    function formatSize(bytes) {{
      if (bytes === 0) return '0 B';
      var k = 1024;
      var sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
      var i = Math.floor(Math.log(bytes) / Math.log(k));
      return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
    }}

    function getFileType(filename) {{
      var ext = filename.split('.').pop().toLowerCase() || '';
      var imageExts = ['png', 'jpg', 'jpeg', 'webp', 'gif', 'svg', 'bmp', 'ico'];
      var videoExts = ['mp4', 'webm', 'ogg', 'mov', 'mkv', 'avi'];
      
      if (imageExts.indexOf(ext) !== -1) return 'image';
      if (videoExts.indexOf(ext) !== -1) return 'video';
      return 'other';
    }}

    function renderFiles(filesList) {{
      var cleanReqPath = window.__INITIAL_DATA__.reqPath.replace(new RegExp('^/|/$', 'g'), '');
      var mobileContainer = document.getElementById('mobile-cards-container');
      var desktopContainer = document.getElementById('desktop-table-body');
      
      var mobileHtml = '';
      var desktopHtml = '';
      
      if (filesList.length === 0) {{
        mobileHtml = '<div class="text-center text-slate-500 py-8 text-xs">No files shared.</div>';
        desktopHtml = '<tr><td colspan="4" style="text-align: center; padding: 2rem; color: var(--muted-foreground); font-size: 0.75rem;">No files shared.</td></tr>';
      }} else {{
        filesList.forEach(function(file) {{
          var suffix = cleanReqPath ? cleanReqPath + '/' + file.name : file.name;
          var relativeUrl = '/' + suffix;
          var fileType = file.isDirectory ? 'other' : getFileType(file.name);
          var typeLabel = file.isDirectory ? 'Folder' : 'File';
          var sizeLabel = file.isDirectory ? '' : formatSize(file.size);
          var desktopSizeLabel = file.isDirectory ? '-' : formatSize(file.size);
          var actionLabel = file.isDirectory ? 'Open Folder' : 'Download';
          var actionLabelDesktop = file.isDirectory ? 'Open' : 'Download';
          
          var btnClass = file.isDirectory ? 'btn-folder' : 'btn-file';
          var actionClassDesktop = file.isDirectory ? 'td-action-folder' : 'td-action-file';
            
          var mobilePreview = '';
          if (file.isDirectory) {{
            mobilePreview = '<span style="font-size: 1.25rem;">📁</span>';
          }} else if (fileType === 'image') {{
            mobilePreview = '<img src="' + relativeUrl + '?preview=true" class="preview-image" alt="" />';
          }} else if (fileType === 'video') {{
            mobilePreview = '<video src="' + relativeUrl + '?preview=true#t=0.5" class="preview-video" preload="metadata" muted></video>';
          }} else {{
            mobilePreview = '<span style="font-size: 1.25rem;">📄</span>';
          }}
          
          mobileHtml += '<div class="file-card" data-name="' + file.name.toLowerCase() + '">' +
            '<div class="file-card-info">' +
              '<div class="preview-box">' + mobilePreview + '</div>' +
              '<div class="file-card-details">' +
                '<h4 class="file-name-title">' + file.name + '</h4>' +
                '<div class="file-meta-row">' +
                  '<span class="badge">' + typeLabel + '</span>' +
                  (sizeLabel ? '<span class="file-size-label">' + sizeLabel + '</span>' : '') +
                '</div>' +
              '</div>' +
            '</div>' +
            '<a href="' + relativeUrl + '" class="action-btn ' + btnClass + '">' + actionLabel + '</a>' +
          '</div>';

          var desktopPreview = '';
          if (file.isDirectory) {{
            desktopPreview = '<span style="font-size: 1.125rem;">📁</span>';
          }} else if (fileType === 'image') {{
            desktopPreview = '<img src="' + relativeUrl + '?preview=true" style="width: 2rem; height: 2rem; object-fit: cover; border-radius: 0.375rem; border: 1px solid var(--border);" alt="" />';
          }} else if (fileType === 'video') {{
            desktopPreview = '<video src="' + relativeUrl + '?preview=true#t=0.5" style="width: 2rem; height: 2rem; object-fit: cover; border-radius: 0.375rem; border: 1px solid var(--border);" preload="metadata" muted></video>';
          }} else {{
            desktopPreview = '<span style="font-size: 1.125rem;">📄</span>';
          }}

          desktopHtml += '<tr class="file-row" data-name="' + file.name.toLowerCase() + '">' +
            '<td><div class="td-name-cell">' +
              '<div style="width: 2rem; height: 2rem; display: flex; align-items: center; justify-content: center; flex-shrink: 0;">' + desktopPreview + '</div>' +
              '<a href="' + relativeUrl + '">' + file.name + '</a>' +
            '</div></td>' +
            '<td class="td-type">' + typeLabel + '</td>' +
            '<td class="td-size">' + desktopSizeLabel + '</td>' +
            '<td class="td-action"><a href="' + relativeUrl + '" class="td-action-link ' + actionClassDesktop + '">' + actionLabelDesktop + '</a></td>' +
          '</tr>';
        }});
      }}
      
      mobileContainer.innerHTML = mobileHtml;
      desktopContainer.innerHTML = desktopHtml;
      
      document.getElementById('files-count-badge').innerHTML =
        '<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" style="color: var(--primary);"><path d="M20 20a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-7.9a2 2 0 0 1-1.69-.9L9.6 3.9A2 2 0 0 0 7.93 3H4a2 2 0 0 0-2 2v13a2 2 0 0 0 2 2Z"></path></svg> ' +
        'Files (' + filesList.length + ')';
    }}

    function fetchFiles() {{
      fetch(window.location.pathname + '?json=true')
        .then(function(res) {{
          if (res.ok) {{
            var contentType = res.headers.get('content-type') || '';
            if (contentType.indexOf('text/html') !== -1) {{
              window.location.reload();
              return;
            }}
            return res.json();
          }}
        }})
        .then(function(data) {{
          if (data) {{
            window.__INITIAL_DATA__.files = data;
            renderFiles(data);
            var query = document.getElementById('search-input').value.toLowerCase();
            if (query) {{ filterFiles(query); }}
          }}
        }})
        .catch(function(e) {{
          console.error('Failed to reload directory files:', e);
        }});
    }}

    function filterFiles(query) {{
      var cards = document.querySelectorAll('.file-card');
      cards.forEach(function(card) {{
        var name = card.getAttribute('data-name');
        if (name.indexOf(query) !== -1) {{
          card.classList.remove('hidden');
        }} else {{
          card.classList.add('hidden');
        }}
      }});

      var rows = document.querySelectorAll('.file-row');
      rows.forEach(function(row) {{
        var name = row.getAttribute('data-name');
        if (name.indexOf(query) !== -1) {{
          row.classList.remove('hidden');
        }} else {{
          row.classList.add('hidden');
        }}
      }});
    }}

    function uploadFiles(fileList) {{
      var formData = new FormData();
      for (var i = 0; i < fileList.length; i++) {{
        formData.append('file', fileList[i]);
      }}

      var progressContainer = document.getElementById('progress-container');
      var progressBar = document.getElementById('progress-bar');
      var progressFilename = document.getElementById('progress-filename');
      var progressPercentage = document.getElementById('progress-percentage');

      progressContainer.classList.remove('hidden');
      progressFilename.textContent = fileList.length === 1 ? fileList[0].name : "Uploading " + fileList.length + " files...";
      progressBar.style.width = '0%';
      progressPercentage.textContent = '0%';

      var xhr = new XMLHttpRequest();
      xhr.open('POST', window.location.pathname, true);

      xhr.upload.onprogress = function(e) {{
        if (e.lengthComputable) {{
          var percent = Math.round((e.loaded / e.total) * 100);
          progressBar.style.width = percent + '%';
          progressPercentage.textContent = percent + '%';
        }}
      }};

      xhr.onload = function() {{
        if (xhr.status === 200 || xhr.status === 303) {{
          fetchFiles();
          setTimeout(function() {{
            progressContainer.classList.add('hidden');
          }}, 1500);
        }} else {{
          alert('Upload failed: ' + xhr.responseText);
          progressContainer.classList.add('hidden');
        }}
      }};

      xhr.onerror = function() {{
        alert('Network error during upload.');
        progressContainer.classList.add('hidden');
      }};

      xhr.send(formData);
    }}

    function init() {{
      renderFiles(window.__INITIAL_DATA__.files);

      document.getElementById('search-input').addEventListener('input', function(e) {{
        filterFiles(e.target.value.toLowerCase());
      }});

      var dropzone = document.getElementById('dropzone');
      var fileInput = document.getElementById('file-input');

      dropzone.addEventListener('click', function() {{ fileInput.click(); }});
      fileInput.addEventListener('change', function(e) {{
        if (e.target.files.length > 0) {{
          uploadFiles(e.target.files);
        }}
      }});

      dropzone.addEventListener('dragover', function(e) {{
        e.preventDefault();
        dropzone.style.borderColor = 'var(--primary)';
        dropzone.style.backgroundColor = 'color-mix(in srgb, var(--primary) 5%, transparent)';
      }});

      dropzone.addEventListener('dragleave', function() {{
        dropzone.style.borderColor = 'var(--border)';
        dropzone.style.backgroundColor = 'color-mix(in srgb, var(--background) 20%, transparent)';
      }});

      dropzone.addEventListener('drop', function(e) {{
        e.preventDefault();
        dropzone.style.borderColor = 'var(--border)';
        dropzone.style.backgroundColor = 'color-mix(in srgb, var(--background) 20%, transparent)';
        if (e.dataTransfer.files.length > 0) {{
          uploadFiles(e.dataTransfer.files);
        }}
      }});

      connectWebSocket();
    }}

    function connectWebSocket() {{
      var protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
      var wsUrl = protocol + '//' + window.location.host;
      var ws;

      try {{
        ws = new WebSocket(wsUrl);
        ws.onmessage = function(event) {{
          try {{
            var msg = JSON.parse(event.data);
            if (msg.type === 'logout') {{
              window.location.reload();
              return;
            }}
            if (msg.type === 'theme-changed') {{
              if (msg.theme === 'dark') {{
                document.documentElement.classList.add('dark');
              }} else {{
                document.documentElement.classList.remove('dark');
              }}
              return;
            }}
            if (msg.type === 'upload-complete' || msg.type === 'directory-changed') {{
              fetchFiles();
            }}
          }} catch (e) {{
            console.error('WebSocket parse failed:', e);
          }}
        }};
        ws.onclose = function() {{
          function checkAuth() {{
            fetch(window.location.pathname + '?json=true')
              .then(function(res) {{
                var contentType = res.headers.get('content-type') || '';
                if (contentType.indexOf('text/html') !== -1) {{
                  window.location.reload();
                }} else {{
                  connectWebSocket();
                }}
              }})
              .catch(function() {{
                setTimeout(checkAuth, 1500);
              }});
          }}
          setTimeout(checkAuth, 1000);
        }};
      }} catch (e) {{
        console.error('WebSocket initial binding failed:', e);
        setTimeout(connectWebSocket, 3000);
      }}
    }}

    window.addEventListener('DOMContentLoaded', init);
  </script>
</body>
</html>"##,
        theme = theme,
        breadcrumb = breadcrumb_html,
        file_count = files.len(),
        req_path_json = req_path_json,
        files_json = files_json,
    )
}

/// Generate the login page HTML for password-protected shares.
pub fn generate_login_html(theme: &str, error: Option<&str>) -> String {
    let error_html = match error {
        Some(msg) => format!(
            r#"<div class="bg-rose-500/10 border border-rose-500/20 text-rose-400 text-xs py-2.5 px-3.5 rounded-md text-center font-medium">{}</div>"#,
            msg
        ),
        None => String::new(),
    };

    format!(
        r#"<!DOCTYPE html>
<html lang="en" class="{theme}">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0, maximum-scale=1.0, user-scalable=no">
  <title>Thlauh Share - Login Required</title>
  <style>
    :root {{
      --background: oklch(1 0 0);
      --foreground: oklch(0.145 0 0);
      --card: oklch(1 0 0);
      --card-foreground: oklch(0.145 0 0);
      --primary: oklch(0.205 0 0);
      --primary-foreground: oklch(0.985 0 0);
      --border: oklch(0.922 0 0);
      --input: oklch(0.922 0 0);
    }}
    .dark {{
      --background: oklch(0.145 0 0);
      --foreground: oklch(0.985 0 0);
      --card: oklch(0.205 0 0);
      --card-foreground: oklch(0.985 0 0);
      --primary: oklch(0.922 0 0);
      --primary-foreground: oklch(0.205 0 0);
      --border: oklch(1 0 0 / 10%);
      --input: oklch(1 0 0 / 10%);
    }}
    html, body {{
      background-color: var(--background);
      color: var(--foreground);
      font-family: system-ui, -apple-system, BlinkMacSystemFont, sans-serif;
    }}
  </style>
</head>
<body style="min-height: 100vh; display: flex; align-items: center; justify-content: center; padding: 1rem; margin: 0;">
  <div style="width: 100%; max-width: 24rem; background: var(--card); border: 1px solid var(--border); border-radius: 0.75rem; padding: 1.5rem; box-shadow: 0 25px 50px -12px rgb(0 0 0 / 0.25); display: flex; flex-direction: column; gap: 1.5rem;">
    <div style="text-align: center;">
      <h2 style="font-size: 1.25rem; font-weight: 700; letter-spacing: -0.025em; margin: 0 0 0.25rem 0;">Login Required</h2>
      <p style="font-size: 0.75rem; color: var(--foreground); opacity: 0.6; margin: 0;">This shared directory is password protected.</p>
    </div>
    
    {error}

    <form method="POST" action="/login" style="display: flex; flex-direction: column; gap: 1rem;">
      <div style="display: flex; flex-direction: column; gap: 0.375rem;">
        <label style="font-size: 10px; font-weight: 700; text-transform: uppercase; letter-spacing: 0.05em; opacity: 0.6;">Password</label>
        <input type="password" name="password" required autofocus placeholder="Enter password..." style="width: 100%; background: var(--background); border: 1px solid var(--input); font-size: 0.875rem; color: var(--foreground); padding: 0.5rem 0.75rem; border-radius: 0.375rem; outline: none; box-sizing: border-box;" />
      </div>
      <button type="submit" style="width: 100%; background: var(--primary); color: var(--primary-foreground); font-weight: 600; padding: 0.625rem; border-radius: 0.375rem; font-size: 0.875rem; border: none; cursor: pointer; box-shadow: 0 10px 15px -3px rgb(0 0 0 / 0.1);">
        Access Directory
      </button>
    </form>
  </div>
</body>
</html>"#,
        theme = theme,
        error = error_html,
    )
}
