import Box from "@mui/material/Box";
import Container from "@mui/material/Container";
import Divider from "@mui/material/Divider";
import List from "@mui/material/List";
import ListItem from "@mui/material/ListItem";
import ListItemText from "@mui/material/ListItemText";
import Typography from "@mui/material/Typography";
import Link from "next/link";

const versions = [
  {
    version: "v1.0.0",
    date: "March 2026",
    changes: [
      "Initial public release",
      "VS Code Extension for seamless terminal orchestration from your editor",
      "Cloud provisioning — one-click managed Mahalaxmi servers",
      "Claude Code provider integration",
      "Ollama provider integration for local LLM support",
    ],
  },
  {
    version: "v0.9.0-beta",
    date: "January 2026",
    changes: [
      "Manager-Worker engine for multi-agent task orchestration",
      "DAG resolver for dependency-aware task scheduling",
      "PTY execution support for full terminal emulation",
    ],
  },
];

export default function ChangelogPage() {
  return (
    <Container maxWidth="md" sx={{ py: 8 }}>
      <Box sx={{ mb: 4 }}>
        <Link
          href="/open-source"
          style={{ color: "inherit", textDecoration: "underline" }}
        >
          ← Back to Open Source
        </Link>
      </Box>

      <Typography variant="h3" component="h1" gutterBottom>
        Changelog
      </Typography>
      <Typography variant="body1" color="text.secondary" sx={{ mb: 6 }}>
        A record of notable changes across Mahalaxmi releases.
      </Typography>

      {versions.map((entry, index) => (
        <Box key={entry.version} sx={{ mb: 6 }}>
          <Typography variant="h5" component="h2" gutterBottom>
            {entry.version} — {entry.date}
          </Typography>
          <Divider sx={{ mb: 2 }} />
          <List dense>
            {entry.changes.map((change) => (
              <ListItem key={change} sx={{ py: 0.5 }}>
                <ListItemText primary={change} />
              </ListItem>
            ))}
          </List>
          {index < versions.length - 1 && <Box sx={{ mb: 2 }} />}
        </Box>
      ))}
    </Container>
  );
}
