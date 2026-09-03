import { render, screen, act } from '@testing-library/react';
import { describe, it, expect } from 'vitest';
import { HashRouter } from 'react-router-dom';
import { App } from './App';

describe('App Component', () => {
  it('renders title and system status', async () => {
    await act(async () => {
      render(
        <HashRouter>
          <App />
        </HashRouter>
      );
    });

    expect(screen.getByText('Life OS Executable Foundation')).toBeInTheDocument();
    expect(screen.getByText('Online')).toBeInTheDocument();
  });

  it('renders IPC ping response state', async () => {
    await act(async () => {
      render(
        <HashRouter>
          <App />
        </HashRouter>
      );
    });

    expect(screen.getByTestId('ping-response')).toBeInTheDocument();
  });
});
