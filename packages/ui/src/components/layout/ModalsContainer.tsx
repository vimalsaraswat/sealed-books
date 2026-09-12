import { useAuth } from "../../context/AuthContext";
import { useLedger } from "../../context/LedgerContext";
import { useModals } from "../../context/ModalContext";

import { AddEntryDialog } from "../domain/AddEntryDialog";
import { TrialBalanceModal } from "../domain/TrialBalanceModal";
import { AuditCertificateModal } from "../domain/AuditCertificateModal";
import { TamperSimulatorDialog } from "../domain/TamperSimulatorDialog";
import { NewPeriodDialog } from "../domain/NewPeriodDialog";
import { AccountsModal } from "../domain/AccountsModal";
import { SealProposalDialog } from "../domain/SealProposalDialog";
import { AuditorReviewModal } from "../domain/AuditorReviewModal";
import { VerificationModal } from "../domain/VerificationModal";
import { TeamModal } from "../domain/TeamModal";
import { CreateOrgModal } from "../domain/CreateOrgModal";

export function ModalsContainer() {
  const { api, activeOrg, userRole, switchOrg } = useAuth();
  const {
    currentPeriod,
    accounts,
    entries,
    statement,
    seal,
    proposalStatement,
    verificationReport,
    isVerifying,
    handleCreateEntry,
    handleCreatePeriod,
    handleCreateAccount,
    handleSimulateTamper,
    handleVerify,
    handleApproveSeal,
    handleDispatchSeal,
    handlePublishSeal,
    handleAuditorApproveAndPublish,
    handleAuditorReject,
    handleLocateOffendingEntry,
  } = useLedger();

  const {
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
  } = useModals();

  return (
    <>
      {currentPeriod && (
        <>
          <AddEntryDialog
            open={isAddEntryOpen}
            onOpenChange={setIsAddEntryOpen}
            period={currentPeriod}
            accounts={accounts}
            onSave={handleCreateEntry}
          />

          <TrialBalanceModal
            open={isTrialBalanceOpen}
            onOpenChange={setIsTrialBalanceOpen}
            period={currentPeriod}
            accounts={accounts}
            entries={entries}
          />

          <AuditCertificateModal
            open={isCertificateOpen}
            onOpenChange={setIsCertificateOpen}
            period={currentPeriod}
            report={verificationReport}
          />

          <TamperSimulatorDialog
            open={isTamperOpen}
            onOpenChange={setIsTamperOpen}
            period={currentPeriod}
            entries={entries}
            onTamper={handleSimulateTamper}
            onTriggerVerify={handleVerify}
          />
        </>
      )}

      <NewPeriodDialog
        open={isNewPeriodOpen}
        onOpenChange={setIsNewPeriodOpen}
        defaultEntity={currentPeriod?.entity || activeOrg?.name}
        onCreate={handleCreatePeriod}
      />

      <AccountsModal
        open={isAccountsOpen}
        onOpenChange={setIsAccountsOpen}
        accounts={accounts}
        onCreateAccount={handleCreateAccount}
      />

      <SealProposalDialog
        open={isCloseModalOpen}
        onOpenChange={setIsCloseModalOpen}
        proposal={proposalStatement}
        seal={seal}
        onApprove={handleApproveSeal}
        onDispatch={handleDispatchSeal}
        onPublish={handlePublishSeal}
      />

      <AuditorReviewModal
        open={isAuditorModalOpen}
        onOpenChange={setIsAuditorModalOpen}
        statementResponse={statement}
        seal={seal}
        onApproveAndPublish={handleAuditorApproveAndPublish}
        onReject={handleAuditorReject}
      />

      <VerificationModal
        open={isVerifyModalOpen}
        onOpenChange={setIsVerifyModalOpen}
        report={verificationReport}
        isLoading={isVerifying}
        onLocateOffendingEntry={handleLocateOffendingEntry}
      />

      <TeamModal
        open={isTeamModalOpen}
        onOpenChange={setIsTeamModalOpen}
        api={api}
        activeOrg={activeOrg}
        userRole={userRole}
      />

      <CreateOrgModal
        open={isCreateOrgOpen}
        onOpenChange={setIsCreateOrgOpen}
        api={api}
        onOrgCreated={async (newOrgId) => {
          await switchOrg(newOrgId);
        }}
      />
    </>
  );
}
