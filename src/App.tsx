import { useState, useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import Header from '@/components/app/Header';
import FilesList from '@/components/app/FilesList';

interface NetworkInterface {
  name: string;
  address: string;
}

interface FileItem {
  name: string;
  isDirectory: boolean;
  size: number;
}

interface ServerInfo {
  path: string;
  serverUrl: string;
  mdnsUrl?: string;
  passwordHash?: string;
}

export interface TransferState {
  direction: 'upload' | 'download';
  filename: string;
  progress: number;
  bytesTransferred: number;
  totalBytes: number;
}

export default function App() {
  const [ipList, setIpList] = useState<NetworkInterface[]>([]);
  const [selectedIp, setSelectedIp] = useState<string>(() => {
    return localStorage.getItem('thlauh_selected_ip') || '';
  });
  const [folderInfo, setFolderInfo] = useState<ServerInfo | null>(null);
  const [files, setFiles] = useState<FileItem[]>([]);
  const [transferState, setTransferState] = useState<TransferState | null>(null);
  const [currentBrowsingPath, setCurrentBrowsingPath] = useState<string>('');
  const [password, setPassword] = useState<string>(() => {
    return localStorage.getItem('thlauh_password') || '';
  });
  const [mdnsHost, setMdnsHost] = useState<string>(() => {
    return localStorage.getItem('thlauh_mdns_host') || '';
  });
  const [configOpen, setConfigOpen] = useState(false);
  const currentPathRef = useRef(currentBrowsingPath);

  useEffect(() => {
    currentPathRef.current = currentBrowsingPath;
  }, [currentBrowsingPath]);

  const [theme, setTheme] = useState<'light' | 'dark'>(() => {
    const saved = localStorage.getItem('thlauh_theme') as 'light' | 'dark' | null;
    if (saved) return saved;
    return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
  });

  // Listen to system prefers-color-scheme changes
  useEffect(() => {
    const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
    const handleThemeChange = (e: MediaQueryListEvent | MediaQueryList) => {
      if (!localStorage.getItem('thlauh_theme')) {
        setTheme(e.matches ? 'dark' : 'light');
      }
    };
    mediaQuery.addEventListener('change', handleThemeChange);
    return () => mediaQuery.removeEventListener('change', handleThemeChange);
  }, []);

  // Update HTML class and sync theme configuration to the file server
  useEffect(() => {
    localStorage.setItem('thlauh_theme', theme);
    if (theme === 'dark') {
      document.documentElement.classList.add('dark');
    } else {
      document.documentElement.classList.remove('dark');
    }
    invoke('cmd_set_theme', { theme });
  }, [theme, folderInfo]);

  // Persist the shared path to enable auto-sharing on restart
  useEffect(() => {
    if (folderInfo) {
      localStorage.setItem('thlauh_shared_path', folderInfo.path);
    } else {
      localStorage.removeItem('thlauh_shared_path');
    }
  }, [folderInfo]);

  // Fetch the active network IP list and auto-start sharing if previously active
  useEffect(() => {
    const fetchIPsAndAutoStart = async () => {
      const ips = await invoke<NetworkInterface[]>('get_ips');
      setIpList(ips);
      
      let ipToUse = '';
      if (ips.length > 0) {
        const savedIp = localStorage.getItem('thlauh_selected_ip');
        const ipExists = ips.some(net => net.address === savedIp);
        ipToUse = savedIp && ipExists ? savedIp : ips[0].address;
        setSelectedIp(ipToUse);
        localStorage.setItem('thlauh_selected_ip', ipToUse);
      }

      const savedPath = localStorage.getItem('thlauh_shared_path');
      if (savedPath && ipToUse) {
        const savedPassword = localStorage.getItem('thlauh_password') || '';
        const savedMdnsHost = localStorage.getItem('thlauh_mdns_host') || '';
        const result = await invoke<ServerInfo | null>('start_server', {
          folderPath: savedPath,
          chosenIp: ipToUse,
          password: savedPassword || null,
          customMdnsHost: savedMdnsHost || null,
        });
        if (result) {
          setFolderInfo(result);
          setCurrentBrowsingPath(savedPath);
          const fileList = await invoke<FileItem[]>('get_files', { path: savedPath });
          setFiles(fileList);
        }
      }
    };
    fetchIPsAndAutoStart();
  }, []);

  // Connect to the WebSocket to monitor transfers and reload directory list
  useEffect(() => {
    if (!folderInfo) {
      setTransferState(null);
      return;
    }

    const wsUrl = folderInfo.serverUrl.replace(/^http:/, 'ws:').replace(/\/$/, '') + '/ws';
    let ws: WebSocket;

    try {
      ws = new WebSocket(wsUrl);

      ws.onmessage = async (event) => {
        try {
          const msg = JSON.parse(event.data);
          
          if (msg.type === 'upload-start') {
            setTransferState({
              direction: 'upload',
              filename: msg.filename,
              progress: 0,
              bytesTransferred: 0,
              totalBytes: msg.totalSize,
            });
          } else if (msg.type === 'upload-progress') {
            setTransferState({
              direction: 'upload',
              filename: msg.filename,
              progress: Math.round((msg.bytesReceived / msg.bytesExpected) * 100),
              bytesTransferred: msg.bytesReceived,
              totalBytes: msg.bytesExpected,
            });
          } else if (msg.type === 'upload-complete') {
            const fileList = await invoke<FileItem[]>('get_files', { path: currentPathRef.current || folderInfo.path });
            setFiles(fileList);
            setTimeout(() => setTransferState(null), 1000);
          } else if (msg.type === 'directory-changed') {
            const fileList = await invoke<FileItem[]>('get_files', { path: currentPathRef.current || folderInfo.path });
            setFiles(fileList);
          } else if (msg.type === 'download-start') {
            setTransferState({
              direction: 'download',
              filename: msg.filename,
              progress: 0,
              bytesTransferred: 0,
              totalBytes: msg.totalSize,
            });
          } else if (msg.type === 'download-progress') {
            setTransferState({
              direction: 'download',
              filename: msg.filename,
              progress: Math.round((msg.bytesDownloaded / msg.totalSize) * 100),
              bytesTransferred: msg.bytesDownloaded,
              totalBytes: msg.totalSize,
            });
          } else if (msg.type === 'download-complete') {
            setTimeout(() => setTransferState(null), 1000);
          }
        } catch (err) {
          console.error('Failed to parse WebSocket message:', err);
        }
      };

      ws.onerror = (err) => {
        console.error('WebSocket connection error:', err);
      };
    } catch (e) {
      console.error('WebSocket connection failed:', e);
    }

    return () => {
      if (ws) ws.close();
    };
  }, [folderInfo]);

  const handleChooseFolder = async () => {
    if (!selectedIp) {
      alert('Please select a valid network interface first.');
      return;
    }

    // Use Tauri's native dialog plugin for folder selection
    const selectedPath = await open({ directory: true, multiple: false });
    
    if (!selectedPath) return;

    const result = await invoke<ServerInfo | null>('start_server', {
      folderPath: selectedPath,
      chosenIp: selectedIp,
      password: password || null,
      customMdnsHost: mdnsHost || null,
    });
    
    if (result) {
      setFolderInfo(result);
      setCurrentBrowsingPath(result.path);
      const fileList = await invoke<FileItem[]>('get_files', { path: result.path });
      setFiles(fileList);
    }
  };

  const handleIpChange = async (value: string) => {
    setSelectedIp(value);
    localStorage.setItem('thlauh_selected_ip', value);
    if (folderInfo) {
      await invoke('cmd_stop_server');
      const result = await invoke<ServerInfo | null>('start_server', {
        folderPath: folderInfo.path,
        chosenIp: value,
        password: password || null,
        customMdnsHost: mdnsHost || null,
      });
      if (result) {
        setFolderInfo(result);
        setCurrentBrowsingPath(folderInfo.path);
      }
    }
  };

  const handleStopSharing = async () => {
    await invoke('cmd_stop_server');
    setFolderInfo(null);
    setCurrentBrowsingPath('');
    setFiles([]);
  };

  const handleNavigate = async (newPath: string) => {
    setCurrentBrowsingPath(newPath);
    const fileList = await invoke<FileItem[]>('get_files', { path: newPath });
    setFiles(fileList);
  };

  const handlePasswordApply = async (newPassword: string) => {
    setPassword(newPassword);
    localStorage.setItem('thlauh_password', newPassword);
    if (folderInfo) {
      await invoke('cmd_stop_server');
      const result = await invoke<ServerInfo | null>('start_server', {
        folderPath: folderInfo.path,
        chosenIp: selectedIp,
        password: newPassword || null,
        customMdnsHost: mdnsHost || null,
      });
      if (result) {
        setFolderInfo(result);
      }
    }
  };

  const handleMdnsHostApply = async (newMdnsHost: string) => {
    setMdnsHost(newMdnsHost);
    localStorage.setItem('thlauh_mdns_host', newMdnsHost);
    if (folderInfo) {
      await invoke('cmd_stop_server');
      const result = await invoke<ServerInfo | null>('start_server', {
        folderPath: folderInfo.path,
        chosenIp: selectedIp,
        password: password || null,
        customMdnsHost: newMdnsHost || null,
      });
      if (result) {
        setFolderInfo(result);
      }
    }
  };

  const handleMdnsHostChange = (newMdnsHost: string) => {
    setMdnsHost(newMdnsHost);
    localStorage.setItem('thlauh_mdns_host', newMdnsHost);
  };

  const handlePasswordChange = (newPassword: string) => {
    setPassword(newPassword);
    localStorage.setItem('thlauh_password', newPassword);
  };

  return (
    <div className="h-screen bg-background text-foreground flex flex-col font-sans antialiased overflow-hidden">
      {/* Header */}
      <Header
        folderInfo={folderInfo}
        ipList={ipList}
        selectedIp={selectedIp}
        onIpChange={handleIpChange}
        onChooseFolder={handleChooseFolder}
        onStopSharing={handleStopSharing}
        password={password}
        onPasswordChange={handlePasswordChange}
        onPasswordApply={handlePasswordApply}
        mdnsHost={mdnsHost}
        onMdnsHostChange={handleMdnsHostChange}
        onMdnsHostApply={handleMdnsHostApply}
        theme={theme}
        onToggleTheme={() => setTheme(prev => prev === 'dark' ? 'light' : 'dark')}
        configOpen={configOpen}
        onConfigOpenChange={setConfigOpen}
      />

      {/* Main Content */}
      <main className="flex-1 p-6 max-w-5xl mx-auto w-full flex flex-col min-h-0 overflow-hidden">
        {/* Files Panel */}
        <div className="flex-1 flex flex-col min-h-0">
          <FilesList 
            files={files} 
            folderInfo={folderInfo} 
            transferState={transferState} 
            currentBrowsingPath={currentBrowsingPath}
            onNavigate={handleNavigate}
            shareUrl={folderInfo ? (password && folderInfo.passwordHash ? `${folderInfo.serverUrl}?token=${folderInfo.passwordHash}` : folderInfo.serverUrl) : ''}
            mdnsUrl={folderInfo && folderInfo.mdnsUrl ? (password && folderInfo.passwordHash ? `${folderInfo.mdnsUrl}?token=${folderInfo.passwordHash}` : folderInfo.mdnsUrl) : ''}
            onOpenConfig={() => setConfigOpen(true)}
          />
        </div>
      </main>
    </div>
  );
}
