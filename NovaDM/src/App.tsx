import { useEffect } from "react";
import { MainLayout } from "./layouts/MainLayout";
import { Downloads } from "./pages/Downloads";
import { History } from "./pages/History";
import { Settings } from "./pages/Settings";
import { useDownloadsStore } from "./store/downloads";
import { useSettingsStore } from "./store/settings";

function App() {
  const currentView = useDownloadsStore((state) => state.currentView);
  const { settings, loadSettings } = useSettingsStore();

  useEffect(() => {
    loadSettings();
  }, [loadSettings]);

  useEffect(() => {
    const applyTheme = () => {
      const root = document.documentElement;
      root.classList.remove('light', 'dark');

      if (settings.theme === 'system') {
        const systemTheme = window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
        root.classList.add(systemTheme);
      } else {
        root.classList.add(settings.theme);
      }
    };

    applyTheme();

    if (settings.theme === 'system') {
      const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
      const listener = () => applyTheme();
      mediaQuery.addEventListener('change', listener);
      return () => mediaQuery.removeEventListener('change', listener);
    }
  }, [settings.theme]);

  const renderView = () => {
    switch (currentView) {
      case 'downloads':
        return <Downloads />;
      case 'history':
        return <History />;
      case 'settings':
        return <Settings />;
      default:
        return <Downloads />;
    }
  };

  return (
    <MainLayout>
      {renderView()}
    </MainLayout>
  );
}

export default App;
