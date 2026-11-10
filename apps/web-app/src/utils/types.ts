export type ExamStatus = "open" | "registered" | "passed" | "locked";

export interface ExamSummary {
  id: string;
  name: string;
  durationMinutes: number;
  questionCount: number;
  passPercent: number;
  status: ExamStatus;
}

export interface Prerequisite {
  name: string;
  kind: "certification" | "identity";
  met: boolean;
}

export interface ExamDetail extends ExamSummary {
  description: string;
  retakeHours: number;
  prerequisites: Prerequisite[];
  attemptId: string | null;
}

export interface Attempt {
  id: string;
  examId: string;
  examName: string;
  identityVerified: boolean;
  desktopConnected: boolean;
}

export interface Certification {
  id: string;
  name: string;
  issuedAt: string;
  scorePercent: number;
}
