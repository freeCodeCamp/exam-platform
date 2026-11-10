import { Flex, Heading, SimpleGrid, Stack } from "@chakra-ui/react";
import { useState } from "react";
import { LuClipboardList, LuSearch, LuSearchX } from "react-icons/lu";

import { AppRoot } from "@/components/app-root";
import { EmptyState } from "@/components/empty-state";
import { ExamCard } from "@/components/exam-card";
import { IconField } from "@/components/icon-field";
import { PageShell } from "@/components/page-shell";
import { ProtectedRoute } from "@/components/protected-route";
import { ROUTES } from "@/utils/routes";
import type { ExamSummary } from "@/utils/types";

function ExamsPage({ exams }: { exams: ExamSummary[] }) {
  const [query, setQuery] = useState("");
  const needle = query.trim().toLowerCase();
  const shown = exams.filter((exam) => exam.name.toLowerCase().includes(needle));

  return (
    <PageShell path={ROUTES.exams}>
      <Stack gap="8">
        <Flex wrap="wrap" align="center" justify="space-between" gap="4">
          <Heading as="h1" fontSize="4xl">
            Exams
          </Heading>
          <Flex flex="0 1 340px">
            <IconField
              id="exam-search"
              label="Search exams"
              icon={LuSearch}
              type="search"
              placeholder="Search"
              h="12"
              bg="bg.panel"
              value={query}
              onChange={(e) => setQuery(e.target.value)}
            />
          </Flex>
        </Flex>

        {exams.length === 0 ? (
          <EmptyState icon={LuClipboardList} title="No exams" />
        ) : shown.length === 0 ? (
          <EmptyState icon={LuSearchX} title="No matching exams" />
        ) : (
          <SimpleGrid as="ul" listStyleType="none" minChildWidth="340px" gap="5">
            {shown.map((exam) => (
              <li key={exam.id}>
                <ExamCard exam={exam} />
              </li>
            ))}
          </SimpleGrid>
        )}
      </Stack>
    </PageShell>
  );
}

export function Exams({ exams = [] }: { exams?: ExamSummary[] }) {
  return (
    <AppRoot>
      <ProtectedRoute>
        <ExamsPage exams={exams} />
      </ProtectedRoute>
    </AppRoot>
  );
}
