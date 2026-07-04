import { Sidebar } from "./components/Sidebar";
import { Downloads } from "./pages/Downloads";
import { History } from "./pages/History";
import { Settings } from "./pages/Settings";
import { useDownloadsStore } from "./store/downloads";

function App() {
  const currentView = useDownloadsStore((state) => state.currentView);

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
    <div className="flex h-screen bg-background">
      <Sidebar />
      {renderView()}
    </div>
  );
}

export default App;
