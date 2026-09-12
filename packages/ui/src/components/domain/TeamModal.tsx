import { useState, useEffect, useCallback } from "react";
import {
  Users,
  UserPlus,
  Mail,
  Shield,
  CheckCircle2,
  AlertCircle,
  Loader2,
} from "lucide-react";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from "../ui/dialog";
import { Button } from "../ui/button";
import { Input } from "../ui/input";
import { Label } from "../ui/label";
import { Badge } from "../ui/badge";
import { Select } from "../ui/select";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "../ui/table";
import { truncateHash } from "../../lib/utils";
import type { ApiClient, Organization, OrgMember } from "../../types";

export interface TeamModalProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  api: ApiClient;
  activeOrg: Organization | null;
  userRole: string;
}

export function TeamModal({
  open,
  onOpenChange,
  api,
  activeOrg,
  userRole,
}: TeamModalProps) {
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
    if (open && activeOrg) {
      loadMembers();
      setStatusMessage(null);
    }
  }, [open, activeOrg, loadMembers]);

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
        inviteName.trim() || undefined,
      );
      setStatusMessage({
        type: "success",
        text: `Successfully invited ${inviteEmail} as ${inviteRole}!`,
      });
      setInviteEmail("");
      setInviteName("");
      await loadMembers();
    } catch (err: any) {
      setStatusMessage({
        type: "error",
        text: err.message || "Failed to send invitation",
      });
    } finally {
      setIsInviting(false);
    }
  };

  const isOwnerOrAdmin = userRole === "owner" || userRole === "admin";

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-2xl bg-card border-border">
        <DialogHeader>
          <div className="flex items-center gap-3">
            <div className="size-9 rounded-lg bg-primary/10 text-primary flex items-center justify-center border border-primary/20">
              <Users className="size-5" />
            </div>
            <div>
              <DialogTitle className="text-lg">Organization Team & Roles</DialogTitle>
              <DialogDescription className="text-xs text-muted-foreground">
                Manage organization members and assign roles for{" "}
                <span className="font-semibold text-foreground">
                  {activeOrg?.name}
                </span>
              </DialogDescription>
            </div>
          </div>
        </DialogHeader>

        <div className="space-y-6 pt-2">
          {/* Invite Form (only for Owner/Admin) */}
          {isOwnerOrAdmin && (
            <form
              onSubmit={handleSendInvite}
              className="p-4 bg-muted/40 border border-border rounded-lg space-y-3"
            >
              <div className="flex items-center gap-2 text-xs font-semibold text-foreground">
                <UserPlus className="size-4 text-primary" />
                <span>Invite New Member / Independent Auditor</span>
              </div>

              <div className="grid grid-cols-1 sm:grid-cols-3 gap-3">
                <div>
                  <Label className="text-[11px] text-muted-foreground">Email Address</Label>
                  <Input
                    type="email"
                    placeholder="colleague@company.com"
                    value={inviteEmail}
                    onChange={(e) => setInviteEmail(e.target.value)}
                    required
                    className="h-8 text-xs bg-background"
                  />
                </div>
                <div>
                  <Label className="text-[11px] text-muted-foreground">Full Name</Label>
                  <Input
                    type="text"
                    placeholder="Jane Doe"
                    value={inviteName}
                    onChange={(e) => setInviteName(e.target.value)}
                    className="h-8 text-xs bg-background"
                  />
                </div>
                <div>
                  <Label className="text-[11px] text-muted-foreground">Assigned Role</Label>
                  <Select
                    value={inviteRole}
                    onChange={(e) => setInviteRole(e.target.value)}
                    className="h-8 text-xs bg-background"
                  >
                    <option value="controller">Controller (Approver 1)</option>
                    <option value="auditor">Independent Auditor (Approver 2)</option>
                    <option value="staff">Staff (Bookkeeper)</option>
                    <option value="admin">Admin / Owner</option>
                  </Select>
                </div>
              </div>

              <div className="flex items-center justify-between pt-1">
                {statusMessage && (
                  <div
                    className={`text-xs flex items-center gap-1.5 ${
                      statusMessage.type === "success"
                        ? "text-emerald-600 dark:text-emerald-400"
                        : "text-destructive"
                    }`}
                  >
                    {statusMessage.type === "success" ? (
                      <CheckCircle2 className="size-4" />
                    ) : (
                      <AlertCircle className="size-4" />
                    )}
                    <span>{statusMessage.text}</span>
                  </div>
                )}
                <div className="ml-auto">
                  <Button
                    type="submit"
                    size="sm"
                    disabled={isInviting || !inviteEmail.trim()}
                    className="h-8 gap-1.5 text-xs"
                  >
                    {isInviting ? (
                      <Loader2 className="size-3.5 animate-spin" />
                    ) : (
                      <Mail className="size-3.5" />
                    )}
                    <span>Send Invitation</span>
                  </Button>
                </div>
              </div>
            </form>
          )}

          {/* Members Table */}
          <div className="border border-border rounded-lg overflow-hidden bg-card">
            <Table>
              <TableHeader>
                <TableRow className="bg-muted/50">
                  <TableHead className="text-xs">User / Member</TableHead>
                  <TableHead className="text-xs">Role</TableHead>
                  <TableHead className="text-xs">Status</TableHead>
                  <TableHead className="text-xs font-mono">Signing Address</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {loading ? (
                  <TableRow>
                    <TableCell colSpan={4} className="text-center py-6 text-xs text-muted-foreground">
                      <Loader2 className="size-5 animate-spin mx-auto text-muted-foreground mb-1" />
                      Loading team members...
                    </TableCell>
                  </TableRow>
                ) : members.length === 0 ? (
                  <TableRow>
                    <TableCell colSpan={4} className="text-center py-6 text-xs text-muted-foreground">
                      No members found.
                    </TableCell>
                  </TableRow>
                ) : (
                  members.map((member) => (
                    <TableRow key={member.id} className="text-xs">
                      <TableCell className="font-medium">
                        <div className="text-foreground">{member.name || "Pending User"}</div>
                        <div className="text-[11px] text-muted-foreground font-mono">
                          {member.email}
                        </div>
                      </TableCell>
                      <TableCell>
                        <Badge
                          variant="outline"
                          className={`text-[10px] uppercase font-mono py-0.5 px-1.5 ${
                            member.role === "controller"
                              ? "bg-primary/10 text-primary border-primary/20"
                              : member.role === "auditor"
                                ? "bg-emerald-500/10 text-emerald-600 dark:text-emerald-400 border-emerald-500/20"
                                : member.role === "owner" || member.role === "admin"
                                  ? "bg-purple-500/10 text-purple-600 dark:text-purple-400 border-purple-500/20"
                                  : "bg-muted text-muted-foreground border-border"
                          }`}
                        >
                          <Shield className="size-2.5 mr-1" />
                          {member.role}
                        </Badge>
                      </TableCell>
                      <TableCell>
                        <Badge
                          variant="secondary"
                          className="text-[10px] font-medium py-0 px-1.5 text-emerald-600 dark:text-emerald-400 bg-emerald-500/10"
                        >
                          {member.status}
                        </Badge>
                      </TableCell>
                      <TableCell className="font-mono text-[11px] text-muted-foreground">
                        {member.eth_address
                          ? truncateHash(member.eth_address, 6, 4)
                          : "Not initialized"}
                      </TableCell>
                    </TableRow>
                  ))
                )}
              </TableBody>
            </Table>
          </div>
        </div>
      </DialogContent>
    </Dialog>
  );
}

