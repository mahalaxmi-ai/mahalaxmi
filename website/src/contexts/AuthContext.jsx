'use client';
import { createContext, useContext, useReducer, useEffect } from 'react';

const AuthContext = createContext(null);

const getInitialState = () => ({
  user: null,
  isLoading: true,
  isAuthenticated: false,
});

function authReducer(state, action) {
  switch (action.type) {
    case 'LOADED':
      return { ...state, isLoading: false, isAuthenticated: true, user: action.payload };
    case 'GUEST':
      return { ...state, isLoading: false, isAuthenticated: false, user: null };
    default:
      return state;
  }
}

export function AuthProvider({ children }) {
  const [state, dispatch] = useReducer(authReducer, null, getInitialState);

  useEffect(() => {
    fetch('/api/auth/me')
      .then((r) => r.json())
      .then((data) => {
        if (data.isAuthenticated && data.user) {
          dispatch({ type: 'LOADED', payload: data.user });
        } else {
          dispatch({ type: 'GUEST' });
        }
      })
      .catch(() => dispatch({ type: 'GUEST' }));
  }, []);

  const login = async (email, password) => {
    try {
      const res = await fetch('/api/auth/login', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ email, password }),
      });
      const data = await res.json();
      if (data.success) {
        dispatch({ type: 'LOADED', payload: data.user });
        return { success: true };
      }
      return { success: false, message: data.message || 'Login failed', code: data.error || '' };
    } catch {
      return { success: false, message: 'An unexpected error occurred. Please try again.' };
    }
  };

  const register = async (userData) => {
    try {
      const res = await fetch('/api/auth/register', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(userData),
      });
      const data = await res.json();
      if (data.success) {
        if (data.user) dispatch({ type: 'LOADED', payload: data.user });
        return { success: true, message: data.message };
      }
      return { success: false, message: data.message || 'Registration failed' };
    } catch {
      return { success: false, message: 'An unexpected error occurred. Please try again.' };
    }
  };

  const logout = async () => {
    try {
      await fetch('/api/auth/logout', { method: 'POST' });
    } catch {
      // ignore
    }
    dispatch({ type: 'GUEST' });
  };

  return (
    <AuthContext.Provider value={{ ...state, login, register, logout }}>
      {children}
    </AuthContext.Provider>
  );
}

export function useAuth() {
  const ctx = useContext(AuthContext);
  if (!ctx) throw new Error('useAuth must be used within AuthProvider');
  return ctx;
}
