import { Settings as SettingsIcon } from 'lucide-react';

export function Settings() {
  return (
    <div className="flex flex-col h-full">
      <header className="border-b border-border bg-card">
        <div className="px-8 py-4">
          <h2 className="text-2xl font-bold text-foreground">Settings</h2>
          <p className="text-sm text-muted-foreground mt-1">
            Configure your preferences
          </p>
        </div>
      </header>

      <main className="flex-1 overflow-auto p-8">
        <div className="flex flex-col items-center justify-center h-full">
          <div className="w-24 h-24 rounded-full bg-secondary flex items-center justify-center mb-6">
            <SettingsIcon className="w-12 h-12 text-muted-foreground" />
          </div>
          <h3 className="text-xl font-semibold text-foreground mb-2">
            Settings coming soon
          </h3>
          <p className="text-muted-foreground text-center max-w-md">
            Configuration options will be available here.
          </p>
        </div>
      </main>
    </div>
  );
}
