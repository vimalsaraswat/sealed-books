import { useState, type FormEvent } from "react";
import {
  ShieldCheck,
  Mail,
  ArrowRight,
  ArrowLeft,
  Loader2,
  AlertCircle,
  Building2,
  User,
} from "lucide-react";
import { Button } from "../ui/button";
import { Input } from "../ui/input";
import { Label } from "../ui/label";
import { Card, CardContent } from "../ui/card";
import {
  InputOTP,
  InputOTPGroup,
  InputOTPSlot,
  InputOTPSeparator,
} from "../ui/input-otp";
import { useAuth } from "../../context/AuthContext";

export function AuthScreen() {
  const { sendOtp, verifyOtp } = useAuth();

  const [step, setStep] = useState<"email" | "otp">("email");
  const [email, setEmail] = useState("");
  const [otpCode, setOtpCode] = useState("");
  const [isNewUser, setIsNewUser] = useState(false);
  const [name, setName] = useState("");
  const [orgName, setOrgName] = useState("");

  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleSendEmail = async (e: FormEvent) => {
    e.preventDefault();
    const cleanEmail = email.trim().toLowerCase();
    if (!cleanEmail) return;

    setLoading(true);
    setError(null);
    try {
      const res = await sendOtp(cleanEmail);
      setIsNewUser(res.is_new_user);
      setStep("otp");
    } catch (err: any) {
      setError(
        err.message || "Failed to send verification code. Please try again.",
      );
    } finally {
      setLoading(false);
    }
  };

  const handleVerifyOtp = async (e: FormEvent) => {
    e.preventDefault();
    const cleanCode = otpCode.trim();
    if (cleanCode.length < 6) return;

    setLoading(true);
    setError(null);
    try {
      await verifyOtp({
        email: email.trim().toLowerCase(),
        code: cleanCode,
        name: name.trim() || undefined,
        organization_name: orgName.trim() || undefined,
      });
    } catch (err: any) {
      setError(
        err.message || "Invalid or expired code. Please verify and try again.",
      );
    } finally {
      setLoading(false);
    }
  };

  const handleResend = async () => {
    setLoading(true);
    setError(null);
    try {
      await sendOtp(email.trim().toLowerCase());
    } catch (err: any) {
      setError(err.message || "Failed to resend code");
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="min-h-screen bg-background text-foreground flex flex-col justify-center items-center px-4 py-16 relative overflow-hidden select-none">
      {/* Ambient gradient glow */}
      <div className="absolute top-1/4 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[600px] h-[350px] bg-primary/10 rounded-full blur-3xl pointer-events-none" />

      <div className="relative w-full max-w-[420px] space-y-6 z-10">
        {/* Brand Header */}
        <div className="text-center space-y-2.5">
          <div className="size-11 rounded-xl bg-primary/10 border border-primary/20 text-primary flex items-center justify-center mx-auto shadow-xs">
            <ShieldCheck className="size-6" />
          </div>
          <div className="space-y-1">
            <h1 className="text-xl font-semibold tracking-tight text-foreground">
              Sealed Books
            </h1>
            <p className="text-xs text-muted-foreground">
              Multi-Party Hedera Consensus Ledger & Audit Vault
            </p>
          </div>
        </div>

        {/* Form Card */}
        <Card className="border-border bg-card/90 shadow-xl backdrop-blur-md">
          <CardContent className="p-6 sm:p-7 space-y-5">
            {error && (
              <div className="p-3 bg-destructive/10 border border-destructive/20 rounded-lg text-xs text-destructive flex items-start gap-2.5">
                <AlertCircle className="size-4 text-destructive shrink-0 mt-0.5" />
                <span className="leading-relaxed">{error}</span>
              </div>
            )}

            {step === "email" ? (
              /* Step 1: Work Email Input */
              <form onSubmit={handleSendEmail} className="space-y-4">
                <div className="space-y-1.5">
                  <Label className="text-xs font-medium text-foreground">
                    Work Email
                  </Label>
                  <div className="relative">
                    <Mail className="size-4 text-muted-foreground absolute left-3.5 top-3" />
                    <Input
                      type="email"
                      placeholder="name@company.com"
                      value={email}
                      onChange={(e) => setEmail(e.target.value)}
                      required
                      autoFocus
                      className="pl-10 h-10 text-xs bg-background border-input text-foreground placeholder:text-muted-foreground focus-visible:ring-primary rounded-lg"
                    />
                  </div>
                </div>

                <Button
                  type="submit"
                  disabled={loading || !email.trim()}
                  className="w-full h-10 text-xs font-medium rounded-lg shadow-xs gap-2 transition-all"
                >
                  {loading ? (
                    <Loader2 className="size-4 animate-spin" />
                  ) : (
                    <>
                      <span>Continue with Email</span>
                      <ArrowRight className="size-3.5" />
                    </>
                  )}
                </Button>

                <p className="text-[11px] text-center text-muted-foreground leading-relaxed pt-1">
                  We will email a one-time secure verification code. New users
                  receive a dedicated cryptographic ledger vault.
                </p>
              </form>
            ) : (
              /* Step 2: 6-Digit Shadcn InputOTP Verification */
              <form onSubmit={handleVerifyOtp} className="space-y-5">
                <div className="space-y-1 text-center pb-1">
                  <div className="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full bg-muted border border-border text-[11px] text-muted-foreground">
                    <span>{email}</span>
                    <button
                      type="button"
                      onClick={() => {
                        setStep("email");
                        setError(null);
                        setOtpCode("");
                      }}
                      className="text-primary hover:underline font-medium ml-1"
                    >
                      Change
                    </button>
                  </div>
                  <p className="text-xs text-muted-foreground pt-2">
                    Enter the 6-digit security code sent to your email
                  </p>
                </div>

                <div className="flex flex-col items-center justify-center space-y-2">
                  <Label className="text-xs font-medium text-foreground self-start">
                    Security Code
                  </Label>
                  <InputOTP
                    maxLength={6}
                    value={otpCode}
                    onChange={(val) => setOtpCode(val)}
                    autoFocus
                  >
                    <InputOTPGroup>
                      <InputOTPSlot index={0} />
                      <InputOTPSlot index={1} />
                      <InputOTPSlot index={2} />
                    </InputOTPGroup>
                    <InputOTPSeparator />
                    <InputOTPGroup>
                      <InputOTPSlot index={3} />
                      <InputOTPSlot index={4} />
                      <InputOTPSlot index={5} />
                    </InputOTPGroup>
                  </InputOTP>
                </div>

                {/* If new user, show optional profile customization */}
                {isNewUser && (
                  <div className="space-y-3 pt-3 border-t border-border">
                    <p className="text-[11px] text-muted-foreground font-medium">
                      Create your organization vault:
                    </p>
                    <div className="space-y-1.5">
                      <Label className="text-[11px] text-muted-foreground">
                        Full Name (Optional)
                      </Label>
                      <div className="relative">
                        <User className="size-3.5 text-muted-foreground absolute left-3 top-2.5" />
                        <Input
                          type="text"
                          placeholder="Your Name"
                          value={name}
                          onChange={(e) => setName(e.target.value)}
                          className="pl-9 h-8 text-xs bg-background border-input text-foreground placeholder:text-muted-foreground rounded-lg"
                        />
                      </div>
                    </div>

                    <div className="space-y-1.5">
                      <Label className="text-[11px] text-muted-foreground">
                        Organization Name (Optional)
                      </Label>
                      <div className="relative">
                        <Building2 className="size-3.5 text-muted-foreground absolute left-3 top-2.5" />
                        <Input
                          type="text"
                          placeholder="e.g. Acme Assurance Corp"
                          value={orgName}
                          onChange={(e) => setOrgName(e.target.value)}
                          className="pl-9 h-8 text-xs bg-background border-input text-foreground placeholder:text-muted-foreground rounded-lg"
                        />
                      </div>
                    </div>
                  </div>
                )}

                <Button
                  type="submit"
                  disabled={loading || otpCode.trim().length < 6}
                  className="w-full h-10 text-xs font-medium rounded-lg shadow-xs gap-2 transition-all mt-1"
                >
                  {loading ? (
                    <Loader2 className="size-4 animate-spin" />
                  ) : (
                    <span>Verify & Enter Vault</span>
                  )}
                </Button>

                <div className="flex items-center justify-between text-xs text-muted-foreground pt-2">
                  <button
                    type="button"
                    onClick={() => {
                      setStep("email");
                      setError(null);
                    }}
                    className="inline-flex items-center gap-1 hover:text-foreground transition-colors"
                  >
                    <ArrowLeft className="size-3" />
                    <span>Back</span>
                  </button>
                  <button
                    type="button"
                    onClick={handleResend}
                    disabled={loading}
                    className="text-primary hover:underline transition-colors"
                  >
                    Resend code
                  </button>
                </div>
              </form>
            )}
          </CardContent>
        </Card>

        {/* Footnote */}
        <div className="text-center space-y-1 text-[11px] text-muted-foreground">
          <div className="flex items-center justify-center gap-1.5 font-mono">
            <span className="size-1.5 rounded-full bg-emerald-500 inline-block animate-pulse" />
            <span>Hedera Consensus Topic 0.0.10462941</span>
          </div>
          <div>Dual-Custody Cryptographic Audit Protocol</div>
        </div>
      </div>
    </div>
  );
}
