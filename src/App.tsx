import { useState, useRef, useEffect, useLayoutEffect } from "react";
import { flushSync } from "react-dom";
import "./App.css";
import ImageConverter from "./ImageConverter";
import type { ConversionRequest } from "./ImageConverter";
import { convertImages } from "./convert";
import logoSvg from "./assets/logo.svg";
import logoDarkSvg from "./assets/logo_dark.svg";
import convertiaSvg from "./assets/convertia.svg";

function App() {
  const [activeView, setActiveView] = useState<"home" | "image-to-image">("home");
  const [isCollapsed, setIsCollapsed] = useState(true);
  const [extendedWidth, setExtendedWidth] = useState<number | null>(null);
  const [isConverting, setIsConverting] = useState(false);
  const [isDarkMode, setIsDarkMode] = useState<boolean>(() => {
    return localStorage.getItem("convertia-theme") === "dark";
  });
  const sidebarRef = useRef<HTMLElement>(null);
  const innerRef = useRef<HTMLDivElement>(null);
  const isTransitioningRef = useRef(false);

  useEffect(() => {
    const l1 = new Image();
    l1.src = logoSvg;
    const l2 = new Image();
    l2.src = logoDarkSvg;
  }, []);

  useLayoutEffect(() => {
    localStorage.setItem("convertia-theme", isDarkMode ? "dark" : "light");
    if (isDarkMode) {
      document.documentElement.classList.add("theme-dark");
    } else {
      document.documentElement.classList.remove("theme-dark");
    }
  }, [isDarkMode]);

  function toggleTheme(event?: React.MouseEvent<HTMLElement>) {
    if (isTransitioningRef.current) return;

    if (
      !document.startViewTransition ||
      window.matchMedia("(prefers-reduced-motion: reduce)").matches
    ) {
      setIsDarkMode((prev) => !prev);
      return;
    }

    isTransitioningRef.current = true;
    document.documentElement.classList.add("theme-transitioning");

    function endThemeTransition() {
      document.documentElement.classList.remove("theme-transitioning");
      isTransitioningRef.current = false;
    }

    let x = event?.clientX;
    let y = event?.clientY;

    if (x === undefined || y === undefined || (x === 0 && y === 0)) {
      if (event?.currentTarget) {
        const rect = event.currentTarget.getBoundingClientRect();
        x = rect.left + rect.width / 2;
        y = rect.top + rect.height / 2;
      } else {
        x = 30;
        y = window.innerHeight - 30;
      }
    }

    const coverageSlack = 1.15;
    const endRadius =
      Math.hypot(
        Math.max(x, window.innerWidth - x),
        Math.max(y, window.innerHeight - y)
      ) * coverageSlack;

    const transition = document.startViewTransition(() => {
      flushSync(() => {
        setIsDarkMode((prev) => !prev);
      });
    });

    transition.ready
      .then(() => {
        const clipPath = [
          `circle(0px at ${x}px ${y}px)`,
          `circle(${endRadius}px at ${x}px ${y}px)`,
        ];
        const anim = document.documentElement.animate(
          {
            clipPath,
          },
          {
            duration: 600,
            easing: "linear",
            pseudoElement: "::view-transition-new(root)",
          }
        );
        void anim.finished.then(endThemeTransition, endThemeTransition);
      })
      .catch(() => undefined);

    void transition.finished.then(endThemeTransition, endThemeTransition);
  }

  async function handleConvertImages(
    request: ConversionRequest
  ): Promise<string | null> {
    setIsConverting(true);
    try {
      return await convertImages(request);
    } finally {
      setIsConverting(false);
    }
  }

  const [isToolsPopoverOpen, setIsToolsPopoverOpen] = useState(false);
  const popoverRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    function measure() {
      if (innerRef.current) {
        const rect = innerRef.current.getBoundingClientRect();
        const measured = rect.width || innerRef.current.offsetWidth || innerRef.current.scrollWidth;
        if (measured > 0) {
          const contentWidth = Math.ceil(measured) + 28;
          const minHeaderWidth = 190;
          setExtendedWidth(Math.max(contentWidth, minHeaderWidth));
        }
      }
    }

    measure();
    if (document.fonts) {
      document.fonts.ready.then(measure);
    }
    if (!innerRef.current) return;
    const observer = new ResizeObserver(() => measure());
    observer.observe(innerRef.current);
    return () => observer.disconnect();
  }, []);

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
    <div className={`app-shell ${isDarkMode ? "theme-dark" : ""}`}>
      <div className="curved-frame">
        <header className="frame-header">
          <img
            src={isDarkMode ? logoDarkSvg : logoSvg}
            alt="Convertia Logo"
            className="frame-logo"
          />
        </header>

        {activeView !== "home" && (
          <button
            type="button"
            className="top-home-arrow-btn"
            style={{
              left: isCollapsed ? 82 : (extendedWidth ? extendedWidth + 24 : 244),
            }}
            onClick={() => setActiveView("home")}
            title="Home"
            aria-label="Back to Home"
          >
            <svg
              width="18"
              height="18"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="2.5"
              strokeLinecap="round"
              strokeLinejoin="round"
            >
              <line x1="19" y1="12" x2="5" y2="12" />
              <polyline points="12 19 5 12 12 5" />
            </svg>
          </button>
        )}

        {activeView === "home" && (
          <div className="locked-hero">
            <h1 className="hero-title">
              <span>
                C
                <img
                  src={convertiaSvg}
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
        )}

        {/* Extendible Left Sidebar */}
        <aside
          ref={sidebarRef}
          className={`curved-sidebar ${isCollapsed ? "collapsed" : "extended"}`}
          style={{ width: isCollapsed ? 58 : (extendedWidth ?? "max-content") }}
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
                    src={convertiaSvg}
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

          {/* Unextended Collapsed Rail */}
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
                  <button
                    className={`tool-item ${activeView === "image-to-image" ? "active" : ""}`}
                    type="button"
                    onClick={() => {
                      setActiveView("image-to-image");
                      setIsToolsPopoverOpen(false);
                    }}
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

            <div className="sidebar-rail-footer">
              <button
                type="button"
                className="sidebar-rail-btn"
                onClick={toggleTheme}
                title={isDarkMode ? "Light Mode" : "Dark Mode"}
                aria-label={isDarkMode ? "Light Mode" : "Dark Mode"}
              >
                {isDarkMode ? (
                  <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                    <circle cx="12" cy="12" r="5" />
                    <line x1="12" y1="1" x2="12" y2="3" />
                    <line x1="12" y1="21" x2="12" y2="23" />
                    <line x1="4.22" y1="4.22" x2="5.64" y2="5.64" />
                    <line x1="18.36" y1="18.36" x2="19.78" y2="19.78" />
                    <line x1="1" y1="12" x2="3" y2="12" />
                    <line x1="21" y1="12" x2="23" y2="12" />
                    <line x1="4.22" y1="19.78" x2="5.64" y2="18.36" />
                    <line x1="18.36" y1="5.64" x2="19.78" y2="4.22" />
                  </svg>
                ) : (
                  <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                    <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z" />
                  </svg>
                )}
              </button>
              <button
                type="button"
                className="sidebar-rail-btn"
                title="Settings"
                aria-label="Settings"
              >
                <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                  <circle cx="12" cy="12" r="3" />
                  <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z" />
                </svg>
              </button>
            </div>
          </div>

          {/* Extended Content */}
          <div className="sidebar-content">
            <div className="sidebar-content-inner" ref={innerRef}>
              <div className="tools-box">
                <h3 className="tools-box-heading">Convertion Tools</h3>
                <button
                  className={`tool-item ${activeView === "image-to-image" ? "active" : ""}`}
                  type="button"
                  onClick={() => {
                    setActiveView("image-to-image");
                  }}
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

              <div className="sidebar-extended-footer">
                <button
                  type="button"
                  className="sidebar-footer-btn"
                  onClick={toggleTheme}
                  title={isDarkMode ? "Light Mode" : "Dark Mode"}
                >
                  {isDarkMode ? (
                    <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                      <circle cx="12" cy="12" r="5" />
                      <line x1="12" y1="1" x2="12" y2="3" />
                      <line x1="12" y1="21" x2="12" y2="23" />
                      <line x1="4.22" y1="4.22" x2="5.64" y2="5.64" />
                      <line x1="18.36" y1="18.36" x2="19.78" y2="19.78" />
                      <line x1="1" y1="12" x2="3" y2="12" />
                      <line x1="21" y1="12" x2="23" y2="12" />
                      <line x1="4.22" y1="19.78" x2="5.64" y2="18.36" />
                      <line x1="18.36" y1="5.64" x2="19.78" y2="4.22" />
                    </svg>
                  ) : (
                    <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                      <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z" />
                    </svg>
                  )}
                  <span>{isDarkMode ? "Light Mode" : "Dark Mode"}</span>
                </button>
                <button
                  type="button"
                  className="sidebar-footer-btn"
                  title="Settings"
                >
                  <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                    <circle cx="12" cy="12" r="3" />
                    <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z" />
                  </svg>
                  <span>Settings</span>
                </button>
              </div>
            </div>
          </div>
        </aside>

        {/* Main Content Area */}
        <main className={`main-viewport view-${activeView}`}>
          {activeView === "image-to-image" && (
            <ImageConverter
              onConvert={handleConvertImages}
              isConverting={isConverting}
            />
          )}
        </main>
      </div>
    </div>
  );
}

export default App;
