import { useState, type FormEvent } from "react";
import {
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
import { BrandLogo } from "../ui/BrandLogo";
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
        <div className="text-center space-y-3">
          <BrandLogo size="lg" className="mx-auto shadow-md" />
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
                    <Mail className="size-4 absolute left-3 top-1/2 -translate-y-1/2 text-muted-foreground" />
                    <Input
                      type="email"
                      required
                      placeholder="auditor@firm.com or cfo@corp.com"
                      value={email}
                      onChange={(e) => setEmail(e.target.value)}
                      className="pl-9 text-xs"
                      autoFocus
                    />
                  </div>
                  <p className="text-[11px] text-muted-foreground">
                    We will dispatch a secure 6-digit verification code.
                  </p>
                </div>

                <Button
                  type="submit"
                  disabled={loading || !email.trim()}
                  className="w-full text-xs font-semibold gap-2 cursor-pointer"
                >
                  {loading ? (
                    <>
                      <Loader2 className="size-3.5 animate-spin" />
                      <span>Sending OTP...</span>
                    </>
                  ) : (
                    <>
                      <span>Continue with Email</span>
                      <ArrowRight className="size-3.5" />
                    </>
                  )}
                </Button>
              </form>
            ) : (
              /* Step 2: OTP Verification & Auto-Onboard */
              <form onSubmit={handleVerifyOtp} className="space-y-5">
                <div className="flex items-center justify-between">
                  <button
                    type="button"
                    onClick={() => {
                      setStep("email");
                      setError(null);
                    }}
                    className="text-xs text-muted-foreground hover:text-foreground flex items-center gap-1 cursor-pointer"
                  >
                    <ArrowLeft className="size-3" />
                    <span>Back</span>
                  </button>
                  <span className="text-[11px] font-mono text-muted-foreground truncate max-w-[200px]">
                    {email}
                  </span>
                </div>

                {isNewUser && (
                  <div className="p-3 bg-primary/5 border border-primary/20 rounded-lg space-y-3">
                    <div className="text-xs font-semibold text-foreground">
                      Welcome! Complete your profile:
                    </div>
                    <div className="space-y-2">
                      <div>
                        <Label className="text-[11px] text-muted-foreground">
                          Your Full Name
                        </Label>
                        <div className="relative mt-1">
                          <User className="size-3.5 absolute left-2.5 top-1/2 -translate-y-1/2 text-muted-foreground" />
                          <Input
                            type="text"
                            placeholder="e.g. Jane Doe"
                            value={name}
                            onChange={(e) => setName(e.target.value)}
                            className="pl-8 text-xs h-8"
                          />
                        </div>
                      </div>
                      <div>
                        <Label className="text-[11px] text-muted-foreground">
                          Organization Name
                        </Label>
                        <div className="relative mt-1">
                          <Building2 className="size-3.5 absolute left-2.5 top-1/2 -translate-y-1/2 text-muted-foreground" />
                          <Input
                            type="text"
                            placeholder="e.g. Acme Corp"
                            value={orgName}
                            onChange={(e) => setOrgName(e.target.value)}
                            className="pl-8 text-xs h-8"
                          />
                        </div>
                      </div>
                    </div>
                  </div>
                )}

                <div className="space-y-2 text-center">
                  <Label className="text-xs font-medium text-foreground">
                    Enter 6-Digit Verification Code
                  </Label>
                  <div className="flex justify-center pt-1">
                    <InputOTP
                      maxLength={6}
                      value={otpCode}
                      onChange={(val) => setOtpCode(val)}
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
                </div>

                <Button
                  type="submit"
                  disabled={loading || otpCode.trim().length < 6}
                  className="w-full text-xs font-semibold gap-2 cursor-pointer"
                >
                  {loading ? (
                    <>
                      <Loader2 className="size-3.5 animate-spin" />
                      <span>Verifying & Signing In...</span>
                    </>
                  ) : (
                    <span>Authenticate Session</span>
                  )}
                </Button>

                <div className="text-center pt-1">
                  <button
                    type="button"
                    onClick={handleResend}
                    disabled={loading}
                    className="text-[11px] text-muted-foreground hover:text-primary underline cursor-pointer disabled:opacity-50"
                  >
                    Didn't receive code? Resend
                  </button>
                </div>
              </form>
            )}
          </CardContent>
        </Card>

        {/* Footnote */}
        <div className="text-center space-y-1">
          <p className="text-[11px] text-muted-foreground font-mono">
            Zero-knowledge ledger session backed by Hedera Consensus Service.
          </p>
        </div>
      </div>
    </div>
  );
}
