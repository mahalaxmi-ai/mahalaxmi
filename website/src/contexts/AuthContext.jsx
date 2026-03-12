"use client";
import { createContext, useContext } from "react";
const ctx = { user: null, isAuthenticated: false, login: async () => {}, logout: async () => {} };
const AuthContext = createContext(ctx);
export function AuthProvider({ children }) { return <AuthContext.Provider value={ctx}>{children}</AuthContext.Provider>; }
export function useAuth() { return useContext(AuthContext); }
