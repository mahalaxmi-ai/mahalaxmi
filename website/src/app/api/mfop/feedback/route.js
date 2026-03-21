import { NextResponse } from 'next/server';
import { mfopSections, mfopMeta } from '@/lib/mfopSpec';

const RECIPIENT = 'support@mahalaxmi.ai';

function buildEmailBody({ name, email, section, type, comment }) {
  const sectionLabel = section
    ? (mfopSections.find((s) => s.id === section)?.title ?? section)
    : 'General (no specific section)';

  return `
MFOP Specification Peer Review Feedback
========================================

From:    ${name} <${email}>
Section: ${sectionLabel}
Type:    ${type}
Date:    ${new Date().toISOString()}

Comment:
--------
${comment}

--
Submitted via https://mahalaxmi.ai/mfop/draft
`.trim();
}

async function sendViaNodemailer(payload) {
  const nodemailer = require('nodemailer');
  const transport = nodemailer.createTransport({
    host: process.env.SMTP_HOST,
    port: parseInt(process.env.SMTP_PORT ?? '587', 10),
    secure: process.env.SMTP_SECURE === 'true',
    auth: { user: process.env.SMTP_USER, pass: process.env.SMTP_PASSWORD },
  });
  const from = process.env.EMAIL_FROM_NAME
    ? `"${process.env.EMAIL_FROM_NAME}" <${process.env.EMAIL_FROM ?? process.env.SMTP_USER}>`
    : (process.env.EMAIL_FROM ?? process.env.SMTP_USER);
  await transport.sendMail({
    from,
    to: RECIPIENT,
    replyTo: `${payload.name} <${payload.email}>`,
    subject: `[MFOP Peer Review] ${payload.type} — ${payload.section || 'General'}`,
    text: buildEmailBody(payload),
  });
}

export async function POST(request) {
  let body;
  try {
    body = await request.json();
  } catch {
    return NextResponse.json({ error: 'Invalid JSON' }, { status: 400 });
  }

  const { name, email, section, type, comment } = body ?? {};

  if (!name?.trim() || !email?.trim() || !comment?.trim()) {
    return NextResponse.json({ error: 'name, email, and comment are required' }, { status: 422 });
  }
  if (!/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email)) {
    return NextResponse.json({ error: 'Invalid email address' }, { status: 422 });
  }

  const payload = {
    name: String(name).slice(0, 200),
    email: String(email).slice(0, 200),
    section: String(section ?? '').slice(0, 100),
    type: String(type ?? 'general').slice(0, 50),
    comment: String(comment).slice(0, 10000),
  };

  try {
    await sendViaNodemailer(payload);
  } catch (err) {
    console.error('[mfop/feedback] Failed to send email:', err);
    return NextResponse.json({ error: 'Failed to send feedback. Please try again.' }, { status: 502 });
  }

  return NextResponse.json({ success: true });
}
