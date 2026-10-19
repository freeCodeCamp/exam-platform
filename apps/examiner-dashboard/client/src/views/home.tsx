import { Text } from "@chakra-ui/react";
import { AppRoot } from "@/components/app-root";
import { PageShell } from "@/components/page-shell";

export function Home() {
  return (
    <AppRoot>
      <PageShell title="Examiner Dashboard">
        <Text>Author, review, promote, and moderate exams.</Text>
      </PageShell>
    </AppRoot>
  );
}
