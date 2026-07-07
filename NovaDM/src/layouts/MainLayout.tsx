import { ReactNode } from 'react';
import { Sidebar } from '../components/Sidebar';
import { DetailsPanel } from '../components/details/DetailsPanel';

interface MainLayoutProps {
  children: ReactNode;
}

export function MainLayout({ children }: MainLayoutProps) {
  return (
    <div className="flex h-screen bg-background">
      <Sidebar />
      <main className="flex-1 overflow-hidden flex">
        <div className="flex-1 overflow-hidden">
          {children}
        </div>
        <DetailsPanel />
      </main>
    </div>
  );
}