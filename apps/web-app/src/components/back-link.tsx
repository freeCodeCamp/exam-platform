import { IconButton } from "@chakra-ui/react";
import { LuArrowLeft } from "react-icons/lu";

export function BackLink({ href, label }: { href: string; label: string }) {
  return (
    <IconButton asChild variant="outline" size="lg" color="fg" flexShrink={0}>
      <a href={href} aria-label={label} title={label}>
        <LuArrowLeft size="20" aria-hidden />
      </a>
    </IconButton>
  );
}
