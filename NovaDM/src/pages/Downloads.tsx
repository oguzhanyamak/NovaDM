import { useState } from 'react';
import { Plus } from 'lucide-react';
import { SectionHeader } from '../components/common/SectionHeader';
import { EmptyState } from '../components/common/EmptyState';
import { DownloadCard } from '../components/download/DownloadCard';
import { NewDownloadDialog } from '../components/NewDownloadDialog';
import { mockDownloads } from '../types/mock-data';

export function Downloads() {
  const [isDialogOpen, setIsDialogOpen] = useState(false);

  return (
    <div className="flex flex-col h-full">
      <header className="border-b border-border bg-card">
        <div className="px-8 py-4">
          <SectionHeader
            title="Downloads"
            description="Manage your downloads"
            action={
              <button 
                onClick={() => setIsDialogOpen(true)}
                className="flex items-center gap-2 px-4 py-2 bg-primary text-primary-foreground rounded-lg hover:bg-primary/90 transition-colors"
              >
                <Plus className="w-4 h-4" />
                New Download
              </button>
            }
          />
        </div>
      </header>

      <main className="flex-1 overflow-auto p-8">
        {mockDownloads.length === 0 ? (
          <EmptyState
            title="No downloads yet"
            description="Start by adding a new download. Your downloads will appear here."
          />
        ) : (
          <div className="space-y-3">
            {mockDownloads.map((download) => (
              <DownloadCard
                key={download.id}
                id={download.id}
                name={download.name}
                url={download.url}
                status={download.status}
                progress={download.progress}
                speed={download.speed}
              />
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
