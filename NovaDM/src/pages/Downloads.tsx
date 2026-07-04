import { useState } from 'react';
import { Download, Plus } from 'lucide-react';
import { useDownloadsStore } from '../store/downloads';
import { NewDownloadDialog } from '../components/NewDownloadDialog';

export function Downloads() {
  const downloads = useDownloadsStore((state) => state.downloads);
  const [isDialogOpen, setIsDialogOpen] = useState(false);

  return (
    <div className="flex flex-col h-full">
      <header className="border-b border-border bg-card">
        <div className="flex items-center justify-between px-8 py-4">
          <div>
            <h2 className="text-2xl font-bold text-foreground">Downloads</h2>
            <p className="text-sm text-muted-foreground mt-1">
              Manage your downloads
            </p>
          </div>
          <button 
            onClick={() => setIsDialogOpen(true)}
            className="flex items-center gap-2 px-4 py-2 bg-primary text-primary-foreground rounded-lg hover:bg-primary/90 transition-colors"
          >
            <Plus className="w-4 h-4" />
            New Download
          </button>
        </div>
      </header>

      <main className="flex-1 overflow-auto p-8">
        {downloads.length === 0 ? (
          <div className="flex flex-col items-center justify-center h-full">
            <div className="w-24 h-24 rounded-full bg-secondary flex items-center justify-center mb-6">
              <Download className="w-12 h-12 text-muted-foreground" />
            </div>
            <h3 className="text-xl font-semibold text-foreground mb-2">
              No downloads yet
            </h3>
            <p className="text-muted-foreground text-center max-w-md">
              Start by adding a new download. Your downloads will appear here.
            </p>
          </div>
        ) : (
          <div className="space-y-3">
            {downloads.map((download) => (
              <div
                key={download.id}
                className="bg-card border border-border rounded-lg p-4 hover:border-primary/50 transition-colors"
              >
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-4">
                    <div className="w-10 h-10 rounded-lg bg-secondary flex items-center justify-center">
                      <Download className="w-5 h-5 text-muted-foreground" />
                    </div>
                    <div>
                      <h4 className="font-medium text-foreground">
                        {download.name}
                      </h4>
                      <p className="text-sm text-muted-foreground">
                        {download.url}
                      </p>
                    </div>
                  </div>
                  <div className="text-right">
                    <p className="text-sm font-medium text-foreground">
                      {download.progress}%
                    </p>
                    <p className="text-xs text-muted-foreground">
                      {download.status}
                    </p>
                  </div>
                </div>
              </div>
            ))}
          </div>
        )}
      </main>

      <NewDownloadDialog 
        open={isDialogOpen} 
        onOpenChange={setIsDialogOpen} 
      />
    </div>
  );
}