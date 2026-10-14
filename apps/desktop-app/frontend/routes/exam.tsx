import { createFileRoute } from "@tanstack/react-router";

// TODO: Might add `/exam/asd123/question/qwe123` so question is its own page
export const Route = createFileRoute("/exam")({
  component: ExamPage,
});

function ExamPage() {
  return <h1>Exam</h1>;
}
