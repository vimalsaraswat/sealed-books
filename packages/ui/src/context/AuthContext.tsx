import {
  createContext,
  useContext,
  useState,
  useEffect,
  useCallback,
  type ReactNode,
} from "react";
import type {
  ApiClient,
  User,
  Organization,
  UserOrgSummary,
  PendingAuditItem,
} from "../types";

export interface AuthContextValue {
  api: ApiClient;
  isAuthenticated: boolean;
  isLoadingAuth: boolean;
  currentUser: User | null;
  activeOrg: Organization | null;
  userRole: string;
  availableOrgs: UserOrgSummary[];
  pendingAudits: PendingAuditItem[];
  sendOtp: (email: string) => Promise<{ status: string; email: string; is_new_user: boolean }>;
  verifyOtp: (payload: { email: string; code: string; name?: string; organization_name?: string }) => Promise<void>;
  login: (
    credential: string | { email?: string; userId?: string; token?: string },
  ) => Promise<void>;
  register: (input: {
    email: string;
    name: string;
    organization_name?: string;
  }) => Promise<void>;
  logout: () => Promise<void>;
  switchOrg: (orgId: string) => Promise<void>;
  refreshAuth: () => Promise<void>;
  refreshPendingAudits: () => Promise<void>;
}

const AuthContext = createContext<AuthContextValue | null>(null);

export interface AuthProviderProps {
  api: ApiClient;
  children: ReactNode;
}

export function AuthProvider({ api, children }: AuthProviderProps) {
  const [isAuthenticated, setIsAuthenticated] = useState(false);
  const [isLoadingAuth, setIsLoadingAuth] = useState(true);
  const [currentUser, setCurrentUser] = useState<User | null>(null);
  const [activeOrg, setActiveOrg] = useState<Organization | null>(null);
  const [userRole, setUserRole] = useState<string>("controller");
  const [availableOrgs, setAvailableOrgs] = useState<UserOrgSummary[]>([]);
  const [pendingAudits, setPendingAudits] = useState<PendingAuditItem[]>([]);

  const refreshPendingAudits = useCallback(async () => {
    try {
      const items = await api.getAuditorPending();
      setPendingAudits(items);
    } catch {
      setPendingAudits([]);
    }
  }, [api]);

  const refreshAuth = useCallback(async () => {
    setIsLoadingAuth(true);
    try {

      const authData = await api.getMe();
      setCurrentUser(authData.user);
      setActiveOrg(authData.active_organization);
      setUserRole(authData.role);
      setAvailableOrgs(authData.available_organizations || []);
      setIsAuthenticated(true);

      if (authData.role === "auditor") {
        await refreshPendingAudits();
      } else {
        setPendingAudits([]);
      }
    } catch {
      // User is not authenticated
      setCurrentUser(null);
      setActiveOrg(null);
      setUserRole("");
      setAvailableOrgs([]);
      setIsAuthenticated(false);
    } finally {
      setIsLoadingAuth(false);
    }
  }, [api, refreshPendingAudits]);

  useEffect(() => {
    refreshAuth();
  }, [refreshAuth]);

  const sendOtp = useCallback(
    async (email: string) => {
      return api.sendOtp(email);
    },
    [api],
  );

  const verifyOtp = useCallback(
    async (payload: { email: string; code: string; name?: string; organization_name?: string }) => {
      const loginRes = await api.verifyOtp(payload);
      setCurrentUser(loginRes.user);
      setActiveOrg(loginRes.active_organization);
      setUserRole(loginRes.role);
      setAvailableOrgs(loginRes.available_organizations || []);
      setIsAuthenticated(true);

      if (loginRes.role === "auditor") {
        await refreshPendingAudits();
      } else {
        setPendingAudits([]);
      }
    },
    [api, refreshPendingAudits],
  );

  const login = useCallback(
    async (
      credential: string | { email?: string; userId?: string; token?: string },
    ) => {
      const loginRes = await api.login(credential);
      setCurrentUser(loginRes.user);
      setActiveOrg(loginRes.active_organization);
      setUserRole(loginRes.role);
      setAvailableOrgs(loginRes.available_organizations || []);
      setIsAuthenticated(true);

      if (loginRes.role === "auditor") {
        await refreshPendingAudits();
      } else {
        setPendingAudits([]);
      }
    },
    [api, refreshPendingAudits],
  );

  const register = useCallback(
    async (input: {
      email: string;
      name: string;
      organization_name?: string;
    }) => {
      setIsLoadingAuth(true);
      try {
        const res = await api.register(input);
        setCurrentUser(res.user);
        setActiveOrg(res.active_organization);
        setUserRole(res.role);
        setAvailableOrgs(res.available_organizations || []);
        setIsAuthenticated(true);
      } finally {
        setIsLoadingAuth(false);
      }
    },
    [api],
  );

  const logout = useCallback(async () => {
    setIsLoadingAuth(true);
    try {
      await api.logout();
    } catch {
      // ignore
    } finally {
      setCurrentUser(null);
      setActiveOrg(null);
      setUserRole("");
      setAvailableOrgs([]);
      setPendingAudits([]);
      setIsAuthenticated(false);
      setIsLoadingAuth(false);
    }
  }, [api]);

  const switchOrg = useCallback(
    async (orgId: string) => {
      try {
        const switchRes = await api.switchOrg(orgId);
        setActiveOrg(switchRes.active_organization);
        setUserRole(switchRes.role);
        setAvailableOrgs(switchRes.available_organizations || []);

        if (switchRes.role === "auditor") {
          await refreshPendingAudits();
        } else {
          setPendingAudits([]);
        }
      } catch (err) {
        console.error("Failed to switch organization:", err);
      }
    },
    [api, refreshPendingAudits],
  );

  return (
    <AuthContext.Provider
      value={{
        api,
        isAuthenticated,
        isLoadingAuth,
        currentUser,
        activeOrg,
        userRole,
        availableOrgs,
        pendingAudits,
        sendOtp,
        verifyOtp,
        login,
        register,
        logout,
        switchOrg,
        refreshAuth,
        refreshPendingAudits,
      }}
    >
      {children}
    </AuthContext.Provider>
  );
}

export function useAuth(): AuthContextValue {
  const context = useContext(AuthContext);
  if (!context) {
    throw new Error("useAuth must be used within an AuthProvider");
  }
  return context;
}
