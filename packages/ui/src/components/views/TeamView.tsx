import { useState, useEffect, useCallback } from "react";
import {
  Users,
  UserPlus,
  CheckCircle2,
  AlertCircle,
  Loader2,
  Building2,
  Plus,
  Check,
  Copy,
} from "lucide-react";
import { Button } from "../ui/button";
import { Input } from "../ui/input";
import { Label } from "../ui/label";
import { Badge } from "../ui/badge";
import { Select } from "../ui/select";
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "../ui/card";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "../ui/table";
import { truncateAddress } from "../../lib/utils";
import { useAuth } from "../../context/AuthContext";
import { useModals } from "../../context/ModalContext";
import type { OrgMember } from "../../types";

export function TeamView() {
  const { api, activeOrg, userRole } = useAuth();
  const { setIsCreateOrgOpen } = useModals();

  const [members, setMembers] = useState<OrgMember[]>([]);
  const [loading, setLoading] = useState(false);
  const [inviteEmail, setInviteEmail] = useState("");
  const [inviteName, setInviteName] = useState("");
  const [inviteRole, setInviteRole] = useState("controller");
  const [isInviting, setIsInviting] = useState(false);
  const [statusMessage, setStatusMessage] = useState<{
    type: "success" | "error";
    text: string;
  } | null>(null);
  const [copiedAddr, setCopiedAddr] = useState<string | null>(null);

  const loadMembers = useCallback(async () => {
    if (!activeOrg) return;
    setLoading(true);
    try {
      const data = await api.getOrgMembers(activeOrg.id);
      setMembers(data);
    } catch (err: any) {
      console.error("Failed to load members:", err);
    } finally {
      setLoading(false);
    }
  }, [api, activeOrg]);

  useEffect(() => {
    if (activeOrg) {
      loadMembers();
    }
  }, [activeOrg, loadMembers]);

  const handleSendInvite = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!activeOrg || !inviteEmail.trim()) return;

    setIsInviting(true);
    setStatusMessage(null);
    try {
      await api.inviteMember(
        activeOrg.id,
        inviteEmail.trim(),
        inviteRole,
        inviteName.trim() || undefined
      );
      setStatusMessage({
        type: "success",
        text: `Successfully added ${inviteEmail} as ${inviteRole}!`,
      });
      setInviteEmail("");
      setInviteName("");
      await loadMembers();
    } catch (err: any) {
      setStatusMessage({
        type: "error",
        text: err.message || "Failed to invite member",
      });
    } finally {
      setIsInviting(false);
    }
  };

  const handleCopy = (addr: string) => {
    navigator.clipboard.writeText(addr);
    setCopiedAddr(addr);
    setTimeout(() => setCopiedAddr(null), 1500);
  };

  if (!activeOrg) {
    return (
      <div className="py-20 text-center space-y-3">
        <Building2 className="size-10 text-muted-foreground mx-auto" />
        <h3 className="text-base font-semibold text-foreground">
          No Organization Selected
        </h3>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* =========================================================
          ORGANIZATION PROFILE HEADER
          ========================================================= */}
      <Card className="border-border shadow-xs">
        <CardContent className="p-5 sm:p-6">
          <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
            <div className="flex items-center gap-3.5">
              <div className="size-11 rounded-lg bg-primary/10 text-primary border border-primary/20 flex items-center justify-center shrink-0">
                <Building2 className="size-6" />
              </div>
              <div>
                <div className="flex items-center gap-2">
                  <h2 className="text-lg font-bold text-foreground">
                    {activeOrg.name}
                  </h2>
                  <Badge variant="outline" className="text-[10px] font-mono uppercase">
                    Your Role: {userRole}
                  </Badge>
                </div>
                <p className="text-xs text-muted-foreground font-mono">
                  Vault ID: {activeOrg.id}
                </p>
              </div>
            </div>

            <Button
              variant="outline"
              size="sm"
              onClick={() => setIsCreateOrgOpen(true)}
              className="h-8 gap-1.5 text-xs"
            >
              <Plus className="size-3.5" />
              <span>Create Another Vault</span>
            </Button>
          </div>
        </CardContent>
      </Card>

      {/* =========================================================
          TEAM MEMBERS & INVITE FORM (2-COLUMN LAYOUT)
          ========================================================= */}
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        {/* Members Roster Table (2 Columns) */}
        <div className="lg:col-span-2 space-y-3">
          <div className="flex items-center justify-between">
            <h3 className="text-sm font-bold text-foreground flex items-center gap-2">
              <Users className="size-4 text-primary" />
              <span>Organization Members ({members.length})</span>
            </h3>
            <span className="text-xs text-muted-foreground font-mono">
              Separation of Duties
            </span>
          </div>

          <div className="rounded-lg border border-border bg-card overflow-hidden shadow-xs">
            <Table>
              <TableHeader className="bg-muted/50">
                <TableRow>
                  <TableHead className="text-xs">Member</TableHead>
                  <TableHead className="w-[120px] text-xs">Role</TableHead>
                  <TableHead className="w-[160px] text-xs">Address / Key</TableHead>
                  <TableHead className="w-[110px] text-xs text-right">Joined</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {loading ? (
                  <TableRow>
                    <TableCell colSpan={4} className="text-center py-8 text-xs text-muted-foreground">
                      <Loader2 className="size-4 animate-spin mx-auto mb-1 text-primary" />
                      Loading roster...
                    </TableCell>
                  </TableRow>
                ) : members.length === 0 ? (
                  <TableRow>
                    <TableCell colSpan={4} className="text-center py-8 text-xs text-muted-foreground">
                      No members found for this organization.
                    </TableCell>
                  </TableRow>
                ) : (
                  members.map((member) => {
                    const isAuditor = member.role === "auditor";
                    const isController = member.role === "controller";
                    return (
                      <TableRow key={member.id} className="hover:bg-muted/40 text-xs">
                        <TableCell className="py-3">
                          <div className="space-y-0.5">
                            <p className="font-semibold text-foreground">
                              {member.name || "Unnamed Member"}
                            </p>
                            <p className="text-[11px] text-muted-foreground font-mono">
                              {member.email}
                            </p>
                          </div>
                        </TableCell>
                        <TableCell className="py-3">
                          <Badge
                            variant={
                              isAuditor
                                ? "outline"
                                : isController
                                ? "default"
                                : "secondary"
                            }
                            className={`capitalize text-[10px] font-mono py-0.5 ${
                              isAuditor
                                ? "border-amber-500/40 text-amber-600 dark:text-amber-400 bg-amber-500/10"
                                : ""
                            }`}
                          >
                            {member.role}
                          </Badge>
                        </TableCell>
                        <TableCell className="py-3 font-mono text-[11px] text-muted-foreground">
                          {member.eth_address ? (
                            <div className="flex items-center gap-1.5">
                              <span>{truncateAddress(member.eth_address)}</span>
                              <button
                                onClick={() => handleCopy(member.eth_address!)}
                                title="Copy public address"
                                className="hover:text-foreground"
                              >
                                {copiedAddr === member.eth_address ? (
                                  <Check className="size-3 text-emerald-500" />
                                ) : (
                                  <Copy className="size-3" />
                                )}
                              </button>
                            </div>
                          ) : (
                            <span>—</span>
                          )}
                        </TableCell>
                        <TableCell className="py-3 text-right font-mono text-[11px] text-muted-foreground">
                          {member.created_at?.slice(0, 10) || "Active"}
                        </TableCell>
                      </TableRow>
                    );
                  })
                )}
              </TableBody>
            </Table>
          </div>
        </div>

        {/* Invite Member Form (Right Col) */}
        <div className="space-y-4">
          <Card className="border-border shadow-xs">
            <CardHeader className="p-4 pb-2">
              <CardTitle className="text-sm font-bold text-foreground flex items-center gap-2">
                <UserPlus className="size-4 text-primary" />
                <span>Invite Member</span>
              </CardTitle>
              <CardDescription className="text-xs text-muted-foreground">
                Add an internal controller or independent external auditor
              </CardDescription>
            </CardHeader>
            <CardContent className="p-4 pt-1">
              <form onSubmit={handleSendInvite} className="space-y-3.5">
                {statusMessage && (
                  <div
                    className={`p-2.5 rounded-md text-xs flex items-center gap-2 ${
                      statusMessage.type === "success"
                        ? "bg-emerald-500/10 text-emerald-600 dark:text-emerald-400 border border-emerald-500/20"
                        : "bg-destructive/10 text-destructive border border-destructive/20"
                    }`}
                  >
                    {statusMessage.type === "success" ? (
                      <CheckCircle2 className="size-4 shrink-0" />
                    ) : (
                      <AlertCircle className="size-4 shrink-0" />
                    )}
                    <span>{statusMessage.text}</span>
                  </div>
                )}

                <div className="space-y-1">
                  <Label htmlFor="inviteEmail" className="text-xs">
                    Email Address *
                  </Label>
                  <Input
                    id="inviteEmail"
                    type="email"
                    placeholder="colleague@firm.com"
                    value={inviteEmail}
                    onChange={(e) => setInviteEmail(e.target.value)}
                    required
                    className="h-8 text-xs"
                  />
                </div>

                <div className="space-y-1">
                  <Label htmlFor="inviteName" className="text-xs">
                    Full Name (Optional)
                  </Label>
                  <Input
                    id="inviteName"
                    type="text"
                    placeholder="e.g. Jane Doe"
                    value={inviteName}
                    onChange={(e) => setInviteName(e.target.value)}
                    className="h-8 text-xs"
                  />
                </div>

                <div className="space-y-1">
                  <Label htmlFor="inviteRole" className="text-xs">
                    Organization Role *
                  </Label>
                  <Select
                    id="inviteRole"
                    value={inviteRole}
                    onChange={(e) => setInviteRole(e.target.value)}
                    className="h-8 text-xs"
                  >
                    <option value="controller">Controller (Propose close & entries)</option>
                    <option value="auditor">Auditor (Independent attestation & co-sign)</option>
                    <option value="staff">Staff (Read & post entries only)</option>
                    <option value="owner">Owner (Full administrative authority)</option>
                  </Select>
                </div>

                <div className="p-2.5 rounded bg-muted/40 border border-border/50 text-[11px] text-muted-foreground space-y-1 font-sans">
                  <p className="font-semibold text-foreground">Separation of Duties:</p>
                  <p>
                    <strong>Auditors</strong> cannot post journal entries. They hold external attestation custody to co-sign period seals.
                  </p>
                </div>

                <Button
                  type="submit"
                  disabled={isInviting || !inviteEmail.trim()}
                  className="w-full h-8 text-xs"
                >
                  {isInviting ? (
                    <>
                      <Loader2 className="size-3.5 animate-spin mr-1.5" />
                      <span>Sending Invitation...</span>
                    </>
                  ) : (
                    <span>Add Member</span>
                  )}
                </Button>
              </form>
            </CardContent>
          </Card>
        </div>
      </div>
    </div>
  );
}
