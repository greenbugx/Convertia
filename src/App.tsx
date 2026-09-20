import { useState, useRef, useCallback, useEffect } from "react";
import "./App.css";

function App() {
  const [isCollapsed, setIsCollapsed] = useState(false);
  const [sidebarWidth, setSidebarWidth] = useState(240);
  const [isDragging, setIsDragging] = useState(false);
  const sidebarRef = useRef<HTMLElement>(null);

  const [isToolsPopoverOpen, setIsToolsPopoverOpen] = useState(false);
  const popoverRef = useRef<HTMLDivElement>(null);

  const startResizing = useCallback((e: React.MouseEvent) => {
    e.preventDefault();
    setIsDragging(true);
  }, []);

  const stopResizing = useCallback(() => {
    setIsDragging(false);
  }, []);

  const resize = useCallback(
    (e: MouseEvent) => {
      if (isDragging && sidebarRef.current) {
        const parentLeft = sidebarRef.current.parentElement?.getBoundingClientRect().left ?? 0;
        const newWidth = e.clientX - parentLeft - 12;
        if (newWidth >= 160 && newWidth <= 380) {
          setSidebarWidth(newWidth);
          if (isCollapsed) {
            setIsCollapsed(false);
            setIsToolsPopoverOpen(false);
          }
        } else if (newWidth < 100) {
          setIsCollapsed(true);
        }
      }
    },
    [isDragging, isCollapsed]
  );

  useEffect(() => {
    if (isDragging) {
      window.addEventListener("mousemove", resize);
      window.addEventListener("mouseup", stopResizing);
    }
    return () => {
      window.removeEventListener("mousemove", resize);
      window.removeEventListener("mouseup", stopResizing);
    };
  }, [isDragging, resize, stopResizing]);

  useEffect(() => {
    function handleClickOutside(event: MouseEvent) {
      if (popoverRef.current && !popoverRef.current.contains(event.target as Node)) {
        setIsToolsPopoverOpen(false);
      }
    }
    if (isToolsPopoverOpen) {
      document.addEventListener("mousedown", handleClickOutside);
    }
    return () => {
      document.removeEventListener("mousedown", handleClickOutside);
    };
  }, [isToolsPopoverOpen]);

  return (
    <div className={`app-shell ${isDragging ? "is-resizing" : ""}`}>
      <div className="curved-frame">
        <header className="frame-header">
          <img src="/logo.svg" alt="Logo" className="frame-logo" />
        </header>

        <div className="locked-hero">
          <h1 className="hero-title">
            <span>
              C
              <img
                src="/convertia.svg"
                alt="o"
                className="inline-convertia-o"
              />
              nvert
            </span>{" "}
            what you{" "}
            <span>
              l
              <span className="heart-o-wrapper">
                o
                <svg
                  className="floating-heart"
                  viewBox="0 0 24 24"
                  fill="#fa2726"
                  aria-hidden="true"
                >
                  <path d="M12 21.35l-1.45-1.32C5.4 15.36 2 12.28 2 8.5 2 5.42 4.42 3 7.5 3c1.74 0 3.41.81 4.5 2.09C13.09 3.81 14.76 3 16.5 3 19.58 3 22 5.42 22 8.5c0 3.78-3.4 6.86-8.55 11.54L12 21.35z" />
                </svg>
              </span>
              ve
            </span>
          </h1>
        </div>

        {/* Extendible Left Sidebar */}
        <aside
          ref={sidebarRef}
          className={`curved-sidebar ${isCollapsed ? "collapsed" : "extended"} ${isDragging ? "dragging" : ""}`}
          style={{ width: isCollapsed ? 58 : sidebarWidth }}
        >
          <div className="sidebar-header">
            {!isCollapsed && <span className="sidebar-title">Menu</span>}
            <button
              className="sidebar-toggle-btn"
              onClick={() => {
                setIsCollapsed(!isCollapsed);
                setIsToolsPopoverOpen(false);
              }}
              title={isCollapsed ? "Extend sidebar" : "Collapse sidebar"}
              aria-label={isCollapsed ? "Extend sidebar" : "Collapse sidebar"}
            >
              {isCollapsed ? (
                <div className="toggle-icon-swap">
                  <img
                    src="/convertia.svg"
                    alt="Convertia"
                    className="convertia-icon"
                  />
                  <svg
                    className="extend-icon"
                    width="18"
                    height="18"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    strokeWidth="2"
                    strokeLinecap="round"
                    strokeLinejoin="round"
                  >
                    <rect x="3" y="3" width="18" height="18" rx="2" ry="2" />
                    <line x1="9" y1="3" x2="9" y2="21" />
                    <polyline points="14 9 17 12 14 15" />
                  </svg>
                </div>
              ) : (
                <svg
                  width="18"
                  height="18"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  strokeWidth="2"
                  strokeLinecap="round"
                  strokeLinejoin="round"
                >
                  <rect x="3" y="3" width="18" height="18" rx="2" ry="2" />
                  <line x1="9" y1="3" x2="9" y2="21" />
                  <polyline points="16 9 13 12 16 15" />
                </svg>
              )}
            </button>
          </div>

          {isCollapsed ? (
            <div className="sidebar-collapsed-content">
              <div className="rail-btn-wrapper" ref={popoverRef}>
                <button
                  className={`sidebar-rail-btn ${isToolsPopoverOpen ? "active" : ""}`}
                  onClick={() => setIsToolsPopoverOpen((prev) => !prev)}
                  title="Convertion Tools"
                  aria-label="Convertion Tools"
                >
                  <svg
                    width="18"
                    height="18"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    strokeWidth="2"
                    strokeLinecap="round"
                    strokeLinejoin="round"
                  >
                    <polyline points="17 1 21 5 17 9" />
                    <path d="M3 5h18" />
                    <polyline points="7 23 3 19 7 15" />
                    <path d="M21 19H3" />
                  </svg>
                </button>

                {isToolsPopoverOpen && (
                  <div className="rail-popover-box">
                    <h3 className="tools-box-heading">Convertion Tools</h3>
                    <button className="tool-item" type="button">
                      <svg
                        width="18"
                        height="18"
                        viewBox="0 0 24 24"
                        fill="none"
                        stroke="currentColor"
                        strokeWidth="2"
                        strokeLinecap="round"
                        strokeLinejoin="round"
                        className="tool-icon"
                      >
                        <rect x="3" y="3" width="18" height="18" rx="2" ry="2" />
                        <circle cx="8.5" cy="8.5" r="1.5" />
                        <polyline points="21 15 16 10 5 21" />
                      </svg>
                      <span className="tool-label">
                        <span>Image</span>
                        <svg
                          width="14"
                          height="14"
                          viewBox="0 0 24 24"
                          fill="none"
                          stroke="currentColor"
                          strokeWidth="2.5"
                          strokeLinecap="round"
                          strokeLinejoin="round"
                          className="bidirectional-arrow"
                          aria-hidden="true"
                        >
                          <line x1="4" y1="12" x2="20" y2="12" />
                          <polyline points="8 8 4 12 8 16" />
                          <polyline points="16 8 20 12 16 16" />
                        </svg>
                        <span>Image</span>
                      </span>
                    </button>
                  </div>
                )}
              </div>
            </div>
          ) : (
            <div className="sidebar-content">
              <div className="tools-box">
                <h3 className="tools-box-heading">Convertion Tools</h3>
                <button className="tool-item" type="button">
                  <svg
                    width="18"
                    height="18"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    strokeWidth="2"
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    className="tool-icon"
                  >
                    <rect x="3" y="3" width="18" height="18" rx="2" ry="2" />
                    <circle cx="8.5" cy="8.5" r="1.5" />
                    <polyline points="21 15 16 10 5 21" />
                  </svg>
                  <span className="tool-label">
                    <span>Image</span>
                    <svg
                      width="14"
                      height="14"
                      viewBox="0 0 24 24"
                      fill="none"
                      stroke="currentColor"
                      strokeWidth="2.5"
                      strokeLinecap="round"
                      strokeLinejoin="round"
                      className="bidirectional-arrow"
                      aria-hidden="true"
                    >
                      <line x1="4" y1="12" x2="20" y2="12" />
                      <polyline points="8 8 4 12 8 16" />
                      <polyline points="16 8 20 12 16 16" />
                    </svg>
                    <span>Image</span>
                  </span>
                </button>
              </div>
            </div>
          )}

          {/* Right edge drag handle for extending */}
          <div
            className="sidebar-resizer"
            onMouseDown={startResizing}
            title="Drag to extend or resize sidebar"
          />
        </aside>

        {/* Main Content Area */}
        <main className="main-viewport">
        </main>
      </div>
    </div>
  );
}

export default App;
