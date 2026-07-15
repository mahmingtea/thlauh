import { Card, CardHeader, CardTitle, CardDescription, CardContent } from '@/components/ui/card';
import { ScrollArea } from '@/components/ui/scroll-area';
import { Badge } from '@/components/ui/badge';
import { Folder, FileText, ArrowUp, QrCode } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover';
import { TransferState } from '@/App';
import { QRCodeSVG } from 'qrcode.react';

interface FileItem {
  name: string;
  isDirectory: boolean;
  size: number;
}

interface FilesListProps {
  files: FileItem[];
  folderInfo: { path: string; serverUrl: string; mdnsUrl?: string; passwordHash?: string } | null;
  transferState: TransferState | null;
  currentBrowsingPath: string;
  onNavigate: (newPath: string) => void;
  shareUrl: string;
  mdnsUrl?: string;
  onOpenConfig: () => void;
}

export default function FilesList({
  files,
  folderInfo,
  transferState,
  currentBrowsingPath,
  onNavigate,
  shareUrl,
  mdnsUrl,
  onOpenConfig
}: FilesListProps) {
  const formatSize = (bytes: number) => {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  };

  const getFileType = (filename: string): 'image' | 'video' | 'other' => {
    const ext = filename.split('.').pop()?.toLowerCase() || '';
    const imageExts = ['png', 'jpg', 'jpeg', 'webp', 'gif', 'svg', 'bmp', 'ico'];
    const videoExts = ['mp4', 'webm', 'ogg', 'mov', 'mkv', 'avi'];

    if (imageExts.includes(ext)) return 'image';
    if (videoExts.includes(ext)) return 'video';
    return 'other';
  };

  // Cross-platform breadcrumbs resolver
  const normalizedRoot = folderInfo ? folderInfo.path.replace(/\\/g, '/') : '';
  const normalizedCurrent = currentBrowsingPath ? currentBrowsingPath.replace(/\\/g, '/') : '';
  const relative = folderInfo && currentBrowsingPath ? normalizedCurrent.slice(normalizedRoot.length) : '';
  const segments = relative.split('/').filter(Boolean);

  const handleRowClick = (file: FileItem) => {
    if (file.isDirectory && folderInfo) {
      const separator = currentBrowsingPath.endsWith('/') || currentBrowsingPath.endsWith('\\') ? '' : '/';
      onNavigate(currentBrowsingPath + separator + file.name);
    }
  };

  const handleGoUp = () => {
    if (folderInfo && currentBrowsingPath !== folderInfo.path) {
      const normalizedPath = currentBrowsingPath.replace(/\\/g, '/');
      const parts = normalizedPath.split('/').filter(Boolean);
      parts.pop();
      const isAbsoluteMac = currentBrowsingPath.startsWith('/');
      const newPath = (isAbsoluteMac ? '/' : '') + parts.join('/');
      onNavigate(newPath);
    }
  };

  const isAtRoot = folderInfo ? currentBrowsingPath === folderInfo.path : true;

  return (
    <Card className="flex flex-col h-full overflow-hidden">
      <CardHeader className="border-b pb-4 shrink-0 flex flex-row items-center justify-between">
        <div className="space-y-1.5">
          <CardTitle className="text-base font-semibold flex items-center gap-2">
            <Folder className="w-4 h-4" />
            Shared Files ({files.length})
          </CardTitle>
          <CardDescription className="text-xs">
            {folderInfo ? "Live files available for sharing" : "Files will appear here once sharing is started."}
          </CardDescription>
        </div>
        {folderInfo && shareUrl && (
          <Popover>
            <PopoverTrigger render={
              <Button variant="outline" size="xs" className="cursor-pointer items-center" title="Show QR Code">
                <QrCode className="w-4 h-4" />Connect
              </Button>
            } />
            <PopoverContent className="w-48 p-3 flex flex-col items-center justify-center gap-2 bg-card border" align="end">
              <p className="text-[10px] text-muted-foreground text-center font-medium">Scan to connect on mobile</p>
              <div className="p-2 bg-white rounded border flex items-center justify-center shadow-sm">
                <QRCodeSVG value={shareUrl} size={128} level="M" />
              </div>
              <div className="text-[10px] text-muted-foreground text-center font-medium w-full space-y-1.5 mt-1 border-t pt-2 border-border/40">
                <p>or enter this URL on your browser:</p>
                <div className="bg-muted/50 py-1 px-1.5 rounded select-all break-all text-[9px] font-mono text-foreground text-left border border-border/20">
                  {shareUrl}
                </div>
                {mdnsUrl && (
                  <>
                    <p className="text-[9px] text-muted-foreground/80 mt-1">or local hostname:</p>
                    <div className="bg-muted/50 py-1 px-1.5 rounded select-all break-all text-[9px] font-mono text-foreground text-left border border-border/20">
                      {mdnsUrl}
                    </div>
                  </>
                )}
              </div>
            </PopoverContent>
          </Popover>
        )}
      </CardHeader>

      {/* Breadcrumbs for folder navigation in desktop app */}
      {folderInfo && (
        <div className="px-4 py-2 border-b bg-muted/20 flex items-center gap-1.5 overflow-x-auto whitespace-nowrap pb-2 text-xs font-semibold text-muted-foreground scrollbar-none shrink-0">
          {!isAtRoot && (
            <>
              <button
                onClick={handleGoUp}
                className="p-1 rounded-md hover:bg-muted text-muted-foreground hover:text-primary transition-colors shrink-0 mr-1"
                title="Go up one folder"
              >
                <ArrowUp className="w-3.5 h-3.5" />
              </button>
              <span className="text-muted-foreground/30 mr-1">|</span>
            </>
          )}
          <button
            onClick={() => onNavigate(folderInfo.path)}
            className="text-primary hover:underline transition-colors shrink-0"
          >
            Root
          </button>
          {segments.map((seg, idx) => {
            const targetPath = normalizedRoot + '/' + segments.slice(0, idx + 1).join('/');
            return (
              <span key={idx} className="flex items-center gap-1.5 shrink-0">
                <span className="text-muted-foreground/45">/</span>
                <button
                  onClick={() => onNavigate(targetPath)}
                  className="text-primary hover:underline transition-colors"
                >
                  {seg}
                </button>
              </span>
            );
          })}
        </div>
      )}

      <CardContent className="p-0 flex-1 overflow-hidden flex flex-col relative">
        {transferState && (
          <div className="p-4 bg-muted/40 border-b space-y-2 shrink-0">
            <div className="flex items-center justify-between text-xs font-semibold">
              <span className="flex items-center gap-1.5 truncate">
                <span className="relative flex h-2 w-2 shrink-0">
                  <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-indigo-400 opacity-75"></span>
                  <span className="relative inline-flex rounded-full h-2 w-2 bg-indigo-500"></span>
                </span>
                <span className="truncate max-w-[200px]">
                  {transferState.direction === 'upload' ? 'Receiving:' : 'Sending:'} {transferState.filename}
                </span>
              </span>
              <span className="font-mono text-muted-foreground shrink-0">{transferState.progress}%</span>
            </div>
            <div className="w-full bg-secondary h-1.5 rounded-full overflow-hidden">
              <div
                className="bg-primary h-1.5 rounded-full transition-all duration-100"
                style={{ width: `${transferState.progress}%` }}
              ></div>
            </div>
            <div className="flex items-center justify-between text-[10px] text-muted-foreground font-mono">
              <span>Transfer Progress</span>
              <span>
                {formatSize(transferState.bytesTransferred)} / {formatSize(transferState.totalBytes)}
              </span>
            </div>
          </div>
        )}

        <div className="flex-1 overflow-hidden relative">
          {files.length > 0 ? (
            <ScrollArea className="h-full">
              <div className="divide-y p-1">
                {files.map((file, index) => {
                  const fileType = file.isDirectory ? 'other' : getFileType(file.name);
                  const fileUrl = folderInfo ? `${folderInfo.serverUrl}/${encodeURIComponent(file.name)}` : '';

                  return (
                    <div
                      key={index}
                      onClick={() => handleRowClick(file)}
                      className={`px-4 py-3 flex items-center justify-between gap-4 text-sm group transition-colors duration-200 ${file.isDirectory ? 'cursor-pointer hover:bg-muted/60' : 'hover:bg-muted/30'}`}
                    >
                      <div className="flex items-center gap-3 min-w-0">
                        <div className="shrink-0 flex items-center justify-center">
                          {file.isDirectory ? (
                            <div className="p-1.5 rounded-md bg-amber-500/10 text-amber-500">
                              <Folder className="w-4 h-4" />
                            </div>
                          ) : fileType === 'image' && fileUrl ? (
                            <img
                              src={`${fileUrl}?preview=true`}
                              alt={file.name}
                              className="w-8 h-8 object-cover rounded-md border border-border"
                            />
                          ) : fileType === 'video' && fileUrl ? (
                            <video
                              src={`${fileUrl}?preview=true#t=0.5`}
                              className="w-8 h-8 object-cover rounded-md border border-border"
                              preload="metadata"
                              muted
                            />
                          ) : (
                            <div className="p-1.5 rounded-md bg-primary/10 text-primary">
                              <FileText className="w-4 h-4" />
                            </div>
                          )}
                        </div>
                        <span className={`truncate ${file.isDirectory ? 'font-semibold text-amber-500/90 group-hover:text-amber-500 group-hover:underline' : 'font-medium'}`}>
                          {file.name}
                        </span>
                      </div>
                      {!file.isDirectory && (
                        <Badge variant="outline" className="font-mono text-[10px] shrink-0">
                          {formatSize(file.size)}
                        </Badge>
                      )}
                    </div>
                  );
                })}
              </div>
            </ScrollArea>
          ) : !folderInfo ? (
            <div className="flex flex-col items-center justify-center h-full p-8 text-center grow">
              <div className="p-4 rounded-full border mb-3 bg-muted/15">
                <Folder className="w-8 h-8 opacity-40" />
              </div>
              <h3 className="font-semibold text-sm text-foreground">Sharing is Offline</h3>
              <p className="text-muted-foreground text-xs mt-1 max-w-[240px] mb-4">
                Configure your network interface and directory to start sharing files.
              </p>
              <Button onClick={onOpenConfig} size="sm" className="cursor-pointer">
                Configure Server
              </Button>
            </div>
          ) : (
            <div className="flex flex-col items-center justify-center h-full p-8 text-center">
              <div className="p-4 rounded-full border mb-3">
                <Folder className="w-8 h-8 opacity-40" />
              </div>
              <h3 className="font-medium text-sm text-muted-foreground">No files shared</h3>
              <p className="text-muted-foreground/60 text-xs mt-1 max-w-[240px]">
                This directory is empty.
              </p>
            </div>
          )}
        </div>
      </CardContent>
    </Card>
  );
}
