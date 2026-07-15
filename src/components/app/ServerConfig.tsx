import { useState } from 'react';
import { Button } from '@/components/ui/button';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Folder, Check, HardDrive, Eye, EyeOff } from 'lucide-react';
import { PopoverClose } from '@/components/ui/popover';

interface NetworkInterface {
  name: string;
  address: string;
}

interface ServerConfigProps {
  ipList: NetworkInterface[];
  selectedIp: string;
  onIpChange: (ip: string) => void;
  folderInfo: { path: string; serverUrl: string; mdnsUrl?: string; passwordHash?: string } | null;
  onChooseFolder: () => void;
  onStopSharing: () => void;
  password?: string;
  onPasswordChange?: (password: string) => void;
  onPasswordApply?: (password: string) => void;
  mdnsHost: string;
  onMdnsHostChange: (val: string) => void;
  onMdnsHostApply: (val: string) => void;
}

export default function ServerConfig({
  ipList,
  selectedIp,
  onIpChange,
  folderInfo,
  onChooseFolder,
  onStopSharing,
  password = '',
  onPasswordChange,
  onPasswordApply,
  mdnsHost = '',
  onMdnsHostChange,
  onMdnsHostApply,
}: ServerConfigProps) {
  const [showPassword, setShowPassword] = useState(false);

  return (
    <div className="p-4 space-y-4 w-80">
      <div className="space-y-1">
        <h3 className="font-semibold text-sm">Server Configuration</h3>
        <p className="text-[11px] text-muted-foreground">
          Configure network adapter and directory.
        </p>
      </div>

      {/* Select Adapter */}
      <div className="space-y-1.5">
        <label className="text-[10px] font-bold uppercase tracking-wider text-muted-foreground block">
          Network Interface
        </label>
        {ipList.length <= 1 ? (
          <div className="flex items-center justify-between border px-2.5 py-1.5 rounded-md bg-muted/40 text-xs text-slate-200 font-medium">
            <span className="truncate">
              {ipList.length === 1 ? `${ipList[0].name} (${ipList[0].address})` : 'No adapters found'}
            </span>
          </div>
        ) : (
          <Select value={selectedIp} onValueChange={(val) => { if (val) onIpChange(val); }}>
            <SelectTrigger className="w-full h-8 text-xs">
              <SelectValue placeholder="Select IP address..." />
            </SelectTrigger>
            <SelectContent>
              {ipList.map((net, idx) => (
                <SelectItem key={idx} value={net.address} className="text-xs">
                  {net.name} ({net.address})
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        )}
      </div>

      {/* Shared Directory Selection / Status */}
      <div className="space-y-1.5">
        <label className="text-[10px] font-bold uppercase tracking-wider text-muted-foreground block">
          Shared Directory
        </label>
        {!folderInfo ? (
          <Button onClick={onChooseFolder} size="sm" className="w-full text-xs h-8 cursor-pointer justify-center">
            <Folder className="w-3.5 h-3.5 mr-1.5" />
            Select Directory
          </Button>
        ) : (
          <div className="flex gap-1.5">
            <div className="flex-1 flex items-center gap-1.5 border px-2.5 py-1.5 rounded-md bg-muted/20 text-xs min-w-0 h-8">
              <HardDrive className="w-3.5 h-3.5 text-muted-foreground shrink-0" />
              <code className="font-mono truncate select-all">{folderInfo.path}</code>
            </div>
            <Button 
              onClick={onChooseFolder} 
              variant="outline"
              size="sm"
              className="h-8 text-xs shrink-0 border border-input px-2.5 cursor-pointer"
              title="Change Directory"
            >
              <Folder className="w-3.5 h-3.5 mr-1.5" />
              Change
            </Button>
          </div>
        )}
      </div>

      {/* Password Protection */}
      <div className="space-y-1.5">
        <label className="text-[10px] font-bold uppercase tracking-wider text-muted-foreground block">
          Password Protection (Optional)
        </label>
        <div className="flex gap-1.5">
          <div className="relative flex-1">
            <input 
              type={showPassword ? 'text' : 'password'}
              placeholder="Set portal password..."
              value={password}
              onChange={(e) => onPasswordChange?.(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === 'Enter') {
                  onPasswordApply?.(password);
                  (e.target as HTMLInputElement).blur();
                }
              }}
              className="w-full bg-background border border-input text-xs text-foreground pl-3 pr-8 py-1.5 rounded-md focus:outline-none focus:border-primary transition-all h-8"
            />
            <button
              type="button"
              onClick={() => setShowPassword(!showPassword)}
              className="absolute right-2.5 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground transition-colors p-0.5"
              title={showPassword ? 'Hide password' : 'Show password'}
            >
              {showPassword ? <EyeOff className="w-3.5 h-3.5" /> : <Eye className="w-3.5 h-3.5" />}
            </button>
          </div>
          {!!folderInfo && (
            <Button
              onClick={() => onPasswordApply?.(password)}
              variant="outline"
              size="icon"
              className="h-8 w-8 shrink-0 border border-input"
              title="Apply password & restart server"
            >
              <Check className="w-3.5 h-3.5 text-emerald-500" />
            </Button>
          )}
        </div>
      </div>

      {/* mDNS Custom Hostname */}
      <div className="space-y-1.5">
        <label className="text-[10px] font-bold uppercase tracking-wider text-muted-foreground block">
          Local Hostname (.local)
        </label>
        <div className="flex gap-1.5">
          <div className="relative flex-1 flex items-center">
            <input 
              type="text"
              placeholder="e.g. thlauh"
              value={mdnsHost}
              onChange={(e) => onMdnsHostChange(e.target.value.replace(/[^a-zA-Z0-9-]/g, ''))}
              onKeyDown={(e) => {
                if (e.key === 'Enter') {
                  onMdnsHostApply(mdnsHost);
                  (e.target as HTMLInputElement).blur();
                }
              }}
              className="w-full bg-background border border-input text-xs text-foreground pl-3 pr-12 py-1.5 rounded-md focus:outline-none focus:border-primary transition-all h-8 font-medium"
            />
            <span className="absolute right-3 text-xs text-muted-foreground font-semibold select-none">
              .local
            </span>
          </div>
          {!!folderInfo && (
            <Button
              onClick={() => onMdnsHostApply(mdnsHost)}
              variant="outline"
              size="icon"
              className="h-8 w-8 shrink-0 border border-input"
              title="Apply hostname & restart server"
            >
              <Check className="w-3.5 h-3.5 text-emerald-500" />
            </Button>
          )}
        </div>
      </div>

      {/* Footer Actions */}
      <div className="flex gap-2 pt-3 border-t border-border/40">
        <PopoverClose render={
          <Button variant="outline" size="sm" className="flex-1 text-xs h-8 cursor-pointer justify-center">
            Close
          </Button>
        } />
        {!!folderInfo && (
          <Button 
            onClick={onStopSharing} 
            variant="destructive"
            size="sm"
            className="flex-1 text-xs h-8 cursor-pointer justify-center"
          >
            Stop Sharing
          </Button>
        )}
      </div>
    </div>
  );
}
