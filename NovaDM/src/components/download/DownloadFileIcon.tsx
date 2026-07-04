import { FileArchive, FileVideo, FileText } from 'lucide-react';
import { cn } from '../../lib/utils';

interface DownloadFileIconProps {
  filename: string;
  className?: string;
}

export function DownloadFileIcon({ filename, className }: DownloadFileIconProps) {
  const extension = filename.split('.').pop()?.toLowerCase() || '';
  
  const getIcon = () => {
    switch (extension) {
      case 'zip':
      case 'rar':
      case '7z':
      case 'tar':
      case 'gz':
        return FileArchive;
      case 'mp4':
      case 'mkv':
      case 'avi':
      case 'mov':
      case 'wmv':
        return FileVideo;
      case 'pdf':
        return FileText;
      default:
        return FileText;
    }
  };

  const Icon = getIcon();

  return (
    <div className={cn(
      'w-10 h-10 rounded-lg bg-secondary flex items-center justify-center',
      className
    )}>
      <Icon className="w-5 h-5 text-muted-foreground" />
    </div>
  );
}