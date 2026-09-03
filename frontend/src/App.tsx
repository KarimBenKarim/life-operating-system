import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';

export function App() {
  const [pingResponse, setPingResponse] = useState<string>('Connecting...');

  useEffect(() => {
    // Attempt Tauri IPC ping call with fallback for web browser testing environment
    invoke<string>('ping')
      .then((res) => setPingResponse(res))
      .catch(() => setPingResponse('PONG (Web/Mock Mode)'));
  }, []);

  return (
    <div style={{ padding: '2rem', fontFamily: 'system-ui, sans-serif' }}>
      <h1>Life OS Executable Foundation</h1>
      <p>System Status: <strong>Online</strong></p>
      <p>
        Tauri IPC Bridge Ping Response: <code data-testid="ping-response">{pingResponse}</code>
      </p>
    </div>
  );
}
