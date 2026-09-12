import { createContext, useContext, useState, type ReactNode } from "react";

export interface ModalContextValue {
  isCloseModalOpen: boolean;
  setIsCloseModalOpen: (open: boolean) => void;
  isAuditorModalOpen: boolean;
  setIsAuditorModalOpen: (open: boolean) => void;
  isVerifyModalOpen: boolean;
  setIsVerifyModalOpen: (open: boolean) => void;
  isAddEntryOpen: boolean;
  setIsAddEntryOpen: (open: boolean) => void;
  isNewPeriodOpen: boolean;
  setIsNewPeriodOpen: (open: boolean) => void;
  isAccountsOpen: boolean;
  setIsAccountsOpen: (open: boolean) => void;
  isTrialBalanceOpen: boolean;
  setIsTrialBalanceOpen: (open: boolean) => void;
  isCertificateOpen: boolean;
  setIsCertificateOpen: (open: boolean) => void;
  isTamperOpen: boolean;
  setIsTamperOpen: (open: boolean) => void;
  isTeamModalOpen: boolean;
  setIsTeamModalOpen: (open: boolean) => void;
  isCreateOrgOpen: boolean;
  setIsCreateOrgOpen: (open: boolean) => void;
}

const ModalContext = createContext<ModalContextValue | null>(null);

export interface ModalProviderProps {
  children: ReactNode;
}

export function ModalProvider({ children }: ModalProviderProps) {
  const [isCloseModalOpen, setIsCloseModalOpen] = useState(false);
  const [isAuditorModalOpen, setIsAuditorModalOpen] = useState(false);
  const [isVerifyModalOpen, setIsVerifyModalOpen] = useState(false);
  const [isAddEntryOpen, setIsAddEntryOpen] = useState(false);
  const [isNewPeriodOpen, setIsNewPeriodOpen] = useState(false);
  const [isAccountsOpen, setIsAccountsOpen] = useState(false);
  const [isTrialBalanceOpen, setIsTrialBalanceOpen] = useState(false);
  const [isCertificateOpen, setIsCertificateOpen] = useState(false);
  const [isTamperOpen, setIsTamperOpen] = useState(false);
  const [isTeamModalOpen, setIsTeamModalOpen] = useState(false);
  const [isCreateOrgOpen, setIsCreateOrgOpen] = useState(false);

  return (
    <ModalContext.Provider
      value={{
        isCloseModalOpen,
        setIsCloseModalOpen,
        isAuditorModalOpen,
        setIsAuditorModalOpen,
        isVerifyModalOpen,
        setIsVerifyModalOpen,
        isAddEntryOpen,
        setIsAddEntryOpen,
        isNewPeriodOpen,
        setIsNewPeriodOpen,
        isAccountsOpen,
        setIsAccountsOpen,
        isTrialBalanceOpen,
        setIsTrialBalanceOpen,
        isCertificateOpen,
        setIsCertificateOpen,
        isTamperOpen,
        setIsTamperOpen,
        isTeamModalOpen,
        setIsTeamModalOpen,
        isCreateOrgOpen,
        setIsCreateOrgOpen,
      }}
    >
      {children}
    </ModalContext.Provider>
  );
}

export function useModals(): ModalContextValue {
  const context = useContext(ModalContext);
  if (!context) {
    throw new Error("useModals must be used within a ModalProvider");
  }
  return context;
}
