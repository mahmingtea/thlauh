import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover';
import ServerConfig from './ServerConfig';
import { Server, Sun, Moon } from 'lucide-react';
import appIcon from '../../../src-tauri/icons/128x128.png';

interface NetworkInterface {
  name: string;
  address: string;
}

interface HeaderProps {
  folderInfo: { path: string; serverUrl: string; mdnsUrl?: string; passwordHash?: string } | null;
  ipList: NetworkInterface[];
  selectedIp: string;
  onIpChange: (ip: string) => void;
  onChooseFolder: () => void;
  onStopSharing: () => void;
  password?: string;
  onPasswordChange?: (password: string) => void;
  onPasswordApply?: (password: string) => void;
  mdnsHost: string;
  onMdnsHostChange: (val: string) => void;
  onMdnsHostApply: (val: string) => void;
  theme: 'light' | 'dark';
  onToggleTheme: () => void;
  configOpen: boolean;
  onConfigOpenChange: (open: boolean) => void;
}

export default function Header({
  folderInfo,
  ipList,
  selectedIp,
  onIpChange,
  onChooseFolder,
  onStopSharing,
  password,
  onPasswordChange,
  onPasswordApply,
  mdnsHost,
  onMdnsHostChange,
  onMdnsHostApply,
  theme,
  onToggleTheme,
  configOpen,
  onConfigOpenChange,
}: HeaderProps) {
  return (
    <header className="border-b px-6 py-4 flex items-center justify-between z-10">
      <div className="flex items-center gap-3">
        <div className="p-1 rounded-lg border">
          <img src={appIcon} className="w-6 h-6 object-contain" alt="Thlauh!" />
        </div>
        <div>
          <h1 className="text-lg font-bold tracking-tight">Thlauh!</h1>
          <p className="text-muted-foreground text-xs">File in share-na awlsam</p>
        </div>
      </div>

      <div className="flex items-center gap-2">
        <Button
          variant="outline"
          size="icon"
          className="h-8 w-8 cursor-pointer"
          onClick={onToggleTheme}
          title={theme === 'dark' ? "Switch to light theme" : "Switch to dark theme"}
        >
          {theme === 'dark' ? <Sun className="w-4 h-4 text-amber-500" /> : <Moon className="w-4 h-4" />}
        </Button>

        <Popover open={configOpen} onOpenChange={onConfigOpenChange}>
          <PopoverTrigger render={
            <Button variant="outline" size="sm" className="flex items-center gap-2 cursor-pointer h-8">
              <Server />
              <span className="text-xs font-medium">Server Config</span>
              <Badge variant={folderInfo ? "default" : "outline"} className="h-5 px-1.5 py-0 flex items-center justify-center">
                <span className={`inline-block w-2 h-2 rounded-full mr-1 ${folderInfo ? 'bg-emerald-500' : 'bg-muted-foreground'}`} />
                {folderInfo ? "Active" : "Offline"}
              </Badge>
            </Button>
          } />
          <PopoverContent className="w-80 p-0" align="end">
            <ServerConfig
              ipList={ipList}
              selectedIp={selectedIp}
              onIpChange={onIpChange}
              folderInfo={folderInfo}
              onChooseFolder={onChooseFolder}
              onStopSharing={onStopSharing}
              password={password}
              onPasswordChange={onPasswordChange}
              onPasswordApply={onPasswordApply}
              mdnsHost={mdnsHost}
              onMdnsHostChange={onMdnsHostChange}
              onMdnsHostApply={onMdnsHostApply}
            />
          </PopoverContent>
        </Popover>
      </div>
    </header>
  );
}
