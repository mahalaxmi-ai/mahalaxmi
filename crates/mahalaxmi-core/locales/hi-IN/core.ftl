# त्रुटियाँ
error-config-file-not-found = { $path } पर कॉन्फ़िगरेशन फ़ाइल नहीं मिली
error-config-parse-failed = कॉन्फ़िगरेशन पार्स करने में विफल: { $reason }
error-config-validation-failed = कॉन्फ़िगरेशन सत्यापन विफल: { $details }
error-locale-not-supported = भाषा "{ $locale }" समर्थित नहीं है
error-log-init-failed = लॉगिंग आरंभ करने में विफल: { $reason }
error-log-dir-create-failed = { $path } पर लॉग निर्देशिका बनाने में विफल: { $reason }
error-app-launch-failed = एप्लिकेशन लॉन्च करने में विफल: { $reason }

# सत्यापन
validation-invalid-log-level = अमान्य लॉग स्तर "{ $level }"। मान्य स्तर: { $valid }
validation-workers-out-of-range = max_concurrent_workers { $min } और { $max } के बीच होना चाहिए, प्राप्त: { $value }
validation-manager-timeout-too-low = manager_timeout_seconds कम से कम { $min } होना चाहिए, प्राप्त: { $value }
validation-worker-timeout-too-low = worker_timeout_seconds कम से कम { $min } होना चाहिए, प्राप्त: { $value }
validation-offline-grace-too-low = offline_grace_days कम से कम { $min } होना चाहिए, प्राप्त: { $value }
validation-invalid-consensus-strategy = अमान्य सहमति रणनीति "{ $value }"। मान्य रणनीतियाँ: { $valid }
validation-invalid-data-directory = data_directory में अमान्य अक्षर हैं
validation-empty-default-provider = providers.default_provider खाली नहीं होना चाहिए
validation-invalid-theme = अमान्य थीम "{ $value }"। मान्य थीम: { $valid }
validation-font-size-out-of-range = terminal_font_size { $min } और { $max } के बीच होना चाहिए, प्राप्त: { $value }
validation-invalid-max-batch-retries = max_batch_retries >= 1 u0939u094bu0928u093e u091au093eu0939u093fu090f, u092au094du0930u093eu092au094du0924: { $value }
validation-invalid-max-total-batches = max_total_batches >= 2 u0939u094bu0928u093e u091au093eu0939u093fu090f, u092au094du0930u093eu092au094du0924: { $value }
validation-invalid-stall-detection-threshold = stall_detection_threshold >= 2 u0939u094bu0928u093e u091au093eu0939u093fu090f, u092au094du0930u093eu092au094du0924: { $value }

# कॉन्फ़िगरेशन
config-loaded-successfully = { $path } से कॉन्फ़िगरेशन लोड किया गया
config-using-default = कोई कॉन्फ़िगरेशन फ़ाइल नहीं मिली, डिफ़ॉल्ट मान का उपयोग किया जा रहा है
config-env-override = पर्यावरण चर { $var } द्वारा कॉन्फ़िगरेशन मान ओवरराइड किया गया
config-env-override-invalid = पर्यावरण चर { $var } का मान "{ $value }" अमान्य है, कॉन्फ़िगर किया गया मान बनाए रखा जा रहा है
config-generated-successfully = { $path } पर डिफ़ॉल्ट कॉन्फ़िगरेशन जनरेट किया गया
config-already-exists = कॉन्फ़िगरेशन फ़ाइल पहले से { $path } पर मौजूद है

# लॉगिंग
logging-initialized = लॉगिंग { $level } स्तर पर आरंभ की गई
logging-rust-log-override = RUST_LOG पर्यावरण चर का पता चला, कॉन्फ़िगर किए गए स्तर को ओवरराइड किया जा रहा है
logging-file-path = लॉग फ़ाइल: { $path }
logging-dir-create-failed-fallback = लॉग निर्देशिका { $path } बनाने में विफल, केवल कंसोल लॉगिंग पर स्विच किया जा रहा है

# PTY
pty-open-failed = छद्म टर्मिनल खोलने में विफल: { $reason }
pty-spawn-failed = PTY में "{ $program }" चलाने में विफल: { $reason }
pty-write-failed = टर्मिनल { $terminal_id } में लिखने में विफल: { $reason }
pty-read-failed = टर्मिनल { $terminal_id } से पढ़ने में विफल: { $reason }
pty-resize-failed = टर्मिनल { $terminal_id } का आकार { $rows }x{ $cols } में बदलने में विफल: { $reason }
pty-wait-failed = टर्मिनल { $terminal_id } की प्रक्रिया स्थिति जाँचने में विफल: { $reason }
pty-kill-failed = टर्मिनल { $terminal_id } की प्रक्रिया समाप्त करने में विफल: { $reason }

# एप्लिकेशन
app-starting = Mahalaxmi v{ $version } प्रारंभ हो रहा है
app-ready = Mahalaxmi तैयार है
app-shutting-down = Mahalaxmi बंद हो रहा है

# प्रमाणपत्र
credential-anthropic-api-key = Claude Code के लिए Anthropic API कुंजी
credential-generic-api-key = { $provider } के लिए API कुंजी
credential-aws-iam-role = { $provider } के लिए AWS IAM भूमिका
credential-oauth-token = { $provider } के लिए OAuth टोकन

# प्रदाता
error-provider-credentials-missing = { $provider } प्रमाणपत्र गायब हैं: पर्यावरण चर { $env_var } सेट नहीं है
error-provider-credentials-invalid = { $provider } प्रमाणपत्र अमान्य हैं: { $reason }
error-provider-not-found = रजिस्ट्री में प्रदाता "{ $provider_id }" नहीं मिला
error-provider-no-default = कोई डिफ़ॉल्ट AI प्रदाता कॉन्फ़िगर नहीं है
error-provider-command-build-failed = { $provider } कमांड बनाने में विफल: { $reason }
provider-registered = प्रदाता "{ $provider }" ID "{ $id }" के साथ पंजीकृत
provider-set-default = डिफ़ॉल्ट प्रदाता "{ $provider }" पर सेट
provider-credentials-valid = { $provider } प्रमाणपत्र सफलतापूर्वक सत्यापित
provider-validating = { $provider } प्रमाणपत्र सत्यापित कर रहे हैं
provider-list-header = पंजीकृत AI प्रदाता

# PTY (विस्तारित)
error-pty-open-failed = PTY खोलने में विफल: { $reason }
error-pty-spawn-failed = PTY में "{ $program }" चलाने में विफल: { $reason }
error-pty-write-failed = टर्मिनल { $terminal_id } में लिखने में विफल: { $reason }
error-pty-read-failed = टर्मिनल { $terminal_id } से पढ़ने में विफल: { $reason }
error-pty-resize-failed = टर्मिनल { $terminal_id } का आकार { $rows }x{ $cols } में बदलने में विफल: { $reason }
error-pty-kill-failed = टर्मिनल { $terminal_id } प्रक्रिया समाप्त करने में विफल: { $reason }
error-pty-wait-failed = टर्मिनल { $terminal_id } प्रक्रिया स्थिति जाँचने में विफल: { $reason }
error-pty-terminal-not-found = टर्मिनल { $terminal_id } नहीं मिला
error-pty-max-concurrent-reached = अधिकतम समवर्ती टर्मिनल ({ $max }) की सीमा पहुँच गई
pty-process-spawned = टर्मिनल { $terminal_id } में प्रक्रिया "{ $program }" प्रारंभ हुई
pty-process-exited = टर्मिनल { $terminal_id } प्रक्रिया कोड { $exit_code } के साथ समाप्त हुई
pty-session-closed = टर्मिनल सत्र { $terminal_id } बंद हुआ
pty-resized = टर्मिनल { $terminal_id } का आकार { $rows }x{ $cols } में बदला गया
pty-reader-eof = टर्मिनल { $terminal_id } रीडर स्ट्रीम के अंत तक पहुँचा
pty-reader-error = टर्मिनल { $terminal_id } रीडर त्रुटि: { $reason }

# ऑर्केस्ट्रेशन त्रुटियाँ
error-orchestration-invalid-transition = { $from } से { $to } में अमान्य स्थिति संक्रमण
error-orchestration-circular-dependency = चक्रीय निर्भरता पाई गई: { $cycle }
error-orchestration-worker-not-found = कार्यकर्ता { $worker_id } कतार में नहीं मिला
error-orchestration-max-retries-exceeded = कार्यकर्ता { $worker_id } ने अधिकतम पुनर्प्रयास सीमा पार कर दी ({ $max_retries })
error-orchestration-no-proposals = कोई प्रबंधक प्रस्ताव प्राप्त नहीं हुआ
error-orchestration-plan-validation-failed = निष्पादन योजना सत्यापन विफल: { $errors }
error-orchestration-consensus-failed = सहमति इंजन विफल: { $reason }
error-orchestration-queue-full = कार्यकर्ता कतार भर गई है (अधिकतम { $max })
error-orchestration-manager-timeout = प्रबंधक { $manager_id } { $timeout }s के बाद समय समाप्त हो गया
error-orchestration-worker-timeout = कार्यकर्ता { $worker_id } { $timeout }s के बाद समय समाप्त हो गया

# ऑर्केस्ट्रेशन जानकारी
orchestration-cycle-started = ऑर्केस्ट्रेशन चक्र { $cycle_id } शुरू हुआ
orchestration-state-changed = स्थिति बदली: { $from } -> { $to }
orchestration-manager-completed = प्रबंधक { $manager_id } ने { $task_count } कार्यों के साथ पूरा किया
orchestration-consensus-reached = सहमति प्राप्त: { $agreed } सहमत, { $dissenting } असहमत
orchestration-plan-created = निष्पादन योजना बनाई गई: { $phases } चरण, { $workers } कार्यकर्ता
orchestration-worker-started = कार्यकर्ता { $worker_id } शुरू: { $task }
orchestration-worker-completed = कार्यकर्ता { $worker_id } { $duration }ms में पूरा हुआ
orchestration-worker-failed = कार्यकर्ता { $worker_id } विफल: { $error }
orchestration-cycle-completed = चक्र { $duration }ms में पूरा हुआ (सफलता दर: { $success_rate })
orchestration-worker-retrying = कार्यकर्ता { $worker_id } पुनर्प्रयास कर रहा है (प्रयास { $attempt }/{ $max })

# पहचान त्रुटियाँ
error-detection-rule-compile-failed = पहचान नियम पैटर्न संकलन विफल: { $reason }
error-detection-no-rules-loaded = कोई पहचान नियम लोड नहीं किए गए
error-detection-invalid-pattern = अमान्य पहचान पैटर्न "{ $pattern }": { $reason }

# पहचान जानकारी
detection-rule-matched = पहचान नियम "{ $rule }" मिला, कार्रवाई: { $action }
detection-rule-cooldown = पहचान नियम "{ $rule }" शीतलन अवधि द्वारा दबाया गया ({ $remaining_ms }ms शेष)
detection-rules-loaded = { $count } पहचान नियम लोड किए गए
detection-provider-rules-applied = प्रदाता { $provider } के लिए { $count } नियम लागू किए गए
detection-error-pattern-detected = त्रुटि पैटर्न पाया गया: "{ $pattern }" ({ $count } बार देखा गया)
detection-root-cause-hypothesis = मूल कारण परिकल्पना: { $category } (विश्वास: { $confidence })
detection-recurring-error = आवर्ती त्रुटि: "{ $message }" ({ $count } बार हुई)
detection-action-executed = कार्रवाई { $action } नियम "{ $rule }" के लिए निष्पादित
detection-cooldowns-reset = { $rule_count } नियमों की शीतलन अवधि रीसेट की गई

# टेम्पलेट त्रुटियाँ
error-template-not-found = टेम्पलेट { $template_id } नहीं मिला
error-template-category-not-found = टेम्पलेट श्रेणी { $category_id } नहीं मिली
error-template-composition-failed = टेम्पलेट संरचना विफल: { $reason }
error-template-include-not-found = शामिल फ़ाइल नहीं मिली: { $path }
error-template-circular-include = चक्रीय शामिल पाया गया (अधिकतम गहराई { $depth } पार)
error-template-placeholder-unresolved = अनसुलझा प्लेसहोल्डर: ${ $placeholder }
error-template-validation-failed = टेम्पलेट सत्यापन { $count } त्रुटियों के साथ विफल
error-template-activation-failed = टेम्पलेट सक्रियण विफल: { $reason }
error-template-catalog-load-failed = टेम्पलेट कैटलॉग लोड करने में विफल: { $path }
error-template-invalid-version = अमान्य टेम्पलेट संस्करण प्रारूप: { $version }

# टेम्पलेट जानकारी
template-catalog-loaded = टेम्पलेट कैटलॉग { $count } टेम्पलेट्स के साथ लोड किया गया
template-activated = टेम्पलेट { $template_id } सफलतापूर्वक सक्रिय किया गया
template-composition-complete = संरचना पूर्ण: { $included } शामिल, { $overridden } ओवरराइड
template-placeholders-resolved = { $count } प्लेसहोल्डर हल किए गए
template-validation-passed = डोमेन { $domain } के लिए टेम्पलेट सत्यापन सफल
template-validation-warnings = टेम्पलेट सत्यापन { $count } चेतावनियों के साथ पूर्ण
template-include-resolved = शामिल हल किया गया: { $path }
template-provider-instructions-injected = प्रदाता { $provider } के लिए निर्देश डाले गए
template-project-config-loaded = प्रोजेक्ट कॉन्फ़िगरेशन { $path } से लोड किया गया
template-domain-validator-registered = डोमेन सत्यापनकर्ता पंजीकृत: { $domain }

# लाइसेंस त्रुटियाँ
error-license-file-not-found = लाइसेंस फ़ाइल नहीं मिली: { $path }
error-license-file-invalid = अमान्य लाइसेंस फ़ाइल ({ $path }): { $reason }
error-license-file-write-failed = लाइसेंस फ़ाइल लिखने में विफल ({ $path }): { $reason }
error-license-signature-invalid = लाइसेंस हस्ताक्षर सत्यापन विफल
error-license-signature-decode-failed = लाइसेंस हस्ताक्षर डिकोड करने में विफल: { $reason }
error-license-serialization-failed = लाइसेंस डेटा क्रमबद्ध करने में विफल: { $reason }
error-license-signing-failed = लाइसेंस पर हस्ताक्षर करने में विफल: { $reason }
error-license-feature-denied = सुविधा '{ $feature }' { $tier } योजना में उपलब्ध नहीं है
error-license-worker-limit = अनुरोधित { $requested } कार्यकर्ता { $tier } योजना की { $limit } सीमा से अधिक हैं
error-license-manager-limit = अनुरोधित { $requested } प्रबंधक { $tier } योजना की { $limit } सीमा से अधिक हैं। अधिक अनलॉक करने के लिए सेटिंग्स → लाइसेंस में अपनी सदस्यता सक्रिय करें।
error-license-category-denied = श्रेणी '{ $category }' के लिए { $required_tier } योजना आवश्यक है (वर्तमान: { $tier })
error-license-fingerprint-hostname = होस्ट नाम निर्धारित करने में विफल: { $reason }
error-license-fingerprint-username = उपयोगकर्ता नाम निर्धारित करने में विफल: { $reason }

# लाइसेंस स्थिति
license-trial-active = परीक्षण लाइसेंस सक्रिय ({ $days } दिन शेष)
license-trial-expiring-soon = परीक्षण अवधि शीघ्र समाप्त हो रही है ({ $days } दिन शेष)
license-trial-expiring-very-soon = परीक्षण अवधि बहुत जल्द समाप्त हो रही है ({ $days } दिन शेष)
license-trial-expired = परीक्षण लाइसेंस की अवधि समाप्त हो गई है
license-expires-later = लाइसेंस { $days } दिनों में समाप्त होगा
license-expires-soon = लाइसेंस शीघ्र समाप्त हो रहा है ({ $days } दिन शेष)
license-expires-very-soon = लाइसेंस बहुत जल्द समाप्त हो रहा है ({ $days } दिन शेष)
license-expires-today = लाइसेंस आज समाप्त हो रहा है
license-grace-period = लाइसेंस समाप्त हो गया है, छूट अवधि सक्रिय है ({ $days } दिन शेष)
license-expired = लाइसेंस की अवधि समाप्त हो गई है

# प्लेटफ़ॉर्म समर्थन — त्रुटि कुंजियाँ
error-platform-unsupported = प्लेटफ़ॉर्म समर्थित नहीं है: { $platform }
error-platform-wsl-not-detected = WSL वातावरण नहीं मिला
error-platform-wsl-path-invalid = WSL अनुवाद के लिए अमान्य पथ: { $path }
error-platform-layout-no-space = पैनल लेआउट के लिए कंटेनर बहुत छोटा है ({ $width }x{ $height })
error-platform-layout-invalid-count = अमान्य पैनल संख्या: { $count }
error-platform-hotkey-registration-failed = हॉटकी पंजीकरण विफल (विरोध): { $shortcut }
error-platform-hotkey-parse-failed = हॉटकी शॉर्टकट पार्स करने में विफल: { $shortcut }
error-platform-shutdown-timeout = प्रक्रिया { $pid } ({ $label }) का शटडाउन समय समाप्त
error-platform-shutdown-failed = प्रक्रिया { $pid } ({ $label }) को बंद करने में विफल
error-platform-shell-not-found = डिफ़ॉल्ट शेल नहीं मिला

# प्लेटफ़ॉर्म समर्थन — सूचना कुंजियाँ
platform-detected = प्लेटफ़ॉर्म पहचाना गया: { $os } ({ $arch })
platform-wsl-detected = WSL पहचाना गया: { $distro } (WSL{ $version })
platform-wsl-path-translated = पथ अनुवादित: { $from } → { $to }
platform-layout-calculated = लेआउट गणना: { $panels } पैनल, { $rows }x{ $cols } ग्रिड
platform-layout-optimized = लेआउट अनुकूलित: { $utilization }% उपयोग
platform-hotkey-registered = हॉटकी पंजीकृत: { $command } → { $shortcut }
platform-hotkey-unregistered = हॉटकी अपंजीकृत: { $command }
platform-shutdown-initiated = { $count } प्रक्रियाओं का शटडाउन आरंभ
platform-shutdown-completed = शटडाउन पूर्ण: { $count } प्रक्रियाएँ, { $duration }ms में
platform-shell-detected = शेल पहचाना गया: { $shell } ({ $path })

# स्मृति त्रुटियाँ
error-memory-not-found = स्मृति प्रविष्टि नहीं मिली: { $id }
error-memory-duplicate = डुप्लिकेट स्मृति प्रविष्टि: { $id }
error-memory-persistence-failed = स्मृति भंडार को सहेजने में विफल: { $reason }
error-memory-load-failed = स्मृति भंडार लोड करने में विफल: { $reason }
error-memory-invalid-confidence = अमान्य विश्वास स्कोर: { $value } (0.0 से 1.0 के बीच होना चाहिए)
error-memory-store-full = स्मृति भंडार भर गया है (अधिकतम { $max } प्रविष्टियाँ)
error-memory-invalid-query = अमान्य स्मृति क्वेरी: { $reason }
error-memory-serialization = स्मृति क्रमबद्धता विफल: { $reason }
error-memory-invalid-entry = अमान्य स्मृति प्रविष्टि: { $reason }
error-memory-session-mismatch = सत्र विसंगति: अपेक्षित { $expected }, प्राप्त { $actual }

# स्मृति सूचना
memory-store-created = सत्र { $session_id } के लिए स्मृति भंडार बनाया गया
memory-entry-added = स्मृति प्रविष्टि जोड़ी गई: { $title } (प्रकार: { $memory_type })
memory-entry-updated = स्मृति प्रविष्टि अपडेट की गई: { $id }
memory-entry-removed = स्मृति प्रविष्टि हटाई गई: { $id }
memory-store-cleared = स्मृति भंडार साफ़ किया गया ({ $count } प्रविष्टियाँ हटाई गईं)
memory-persisted = स्मृति भंडार { $path } पर सहेजा गया
memory-loaded = स्मृति भंडार { $path } से लोड किया गया ({ $count } प्रविष्टियाँ)
memory-query-executed = स्मृति क्वेरी ने { $count } परिणाम दिए
memory-injected = { $count } स्मृतियाँ इंजेक्ट की गईं ({ $tokens } टोकन)
memory-stats = स्मृति आँकड़े: { $total } प्रविष्टियाँ, औसत विश्वास { $avg_confidence }

# अनुक्रमण त्रुटियाँ
error-indexing-parse-failed = { $file } का विश्लेषण करने में विफल: { $reason }
error-indexing-file-read-failed = फ़ाइल { $file } पढ़ने में विफल: { $reason }
error-indexing-unsupported-language = फ़ाइल एक्सटेंशन के लिए असमर्थित भाषा: { $extension }
error-indexing-extraction-failed = { $file } के लिए सिंबल निष्कर्षण विफल: { $reason }
error-indexing-graph-cycle-detected = निर्भरता चक्र पाया गया: { $files }
error-indexing-fingerprint-failed = { $file } के लिए फिंगरप्रिंट गणना विफल: { $reason }
error-indexing-build-failed = अनुक्रमणिका निर्माण विफल: { $reason }
error-indexing-update-failed = वृद्धिशील अद्यतन विफल: { $reason }

# अनुक्रमण सूचना
indexing-file-indexed = फ़ाइल अनुक्रमित: { $file } ({ $language })
indexing-symbols-extracted = { $file } से { $count } सिंबल निकाले गए
indexing-graph-built = निर्भरता ग्राफ़ बनाया गया: { $files } फ़ाइलें, { $edges } किनारे
indexing-ranking-computed = { $symbols } सिंबल के लिए रैंकिंग गणना की गई
indexing-repomap-generated = रिपॉजिटरी मैप बनाया गया: { $symbols } सिंबल, { $tokens } टोकन
indexing-index-built = कोडबेस अनुक्रमणिका बनाई गई: { $files } फ़ाइलें, { $symbols } सिंबल
indexing-incremental-update = वृद्धिशील अद्यतन: { $added } जोड़े गए, { $modified } संशोधित, { $removed } हटाए गए
indexing-language-registered = भाषा पंजीकृत: { $language }

# संदर्भ त्रुटियाँ
error-context-budget-exceeded = संदर्भ टोकन बजट पार हुआ: उपयोग किया { $used }, बजट { $budget }
error-context-invalid-allocations = बजट आवंटन का योग <= 1.0 होना चाहिए, प्राप्त { $sum }
error-context-build-failed = कार्य { $task_id } के लिए संदर्भ निर्माण विफल: { $reason }
error-context-invalid-format = अमान्य संदर्भ प्रारूप: { $format }

# संदर्भ जानकारी
context-budget-allocated = टोकन बजट आवंटित: { $total } टोकन ({ $repo_map } रिपॉजिटरी मैप, { $files } फ़ाइलें, { $memory } मेमोरी, { $task } कार्य)
context-files-scored = { $count } फ़ाइलों की प्रासंगिकता मूल्यांकित (शीर्ष: { $top_file })
context-chunks-created = { $count } कोड खंड बनाए गए ({ $tokens } टोकन)
context-assembled = संदर्भ संयोजित: { $sections } खंड, { $budget } बजट में से { $tokens } टोकन उपयोग
context-injected = वर्कर { $worker_id } के लिए संदर्भ इंजेक्ट किया गया ({ $tokens } टोकन, { $files } फ़ाइलें)
context-skipped = संदर्भ तैयारी छोड़ दी गई: { $reason }

# MCP त्रुटियाँ
error-mcp-parse-failed = JSON-RPC संदेश का विश्लेषण विफल: { $reason }
error-mcp-invalid-request = अमान्य JSON-RPC अनुरोध: { $reason }
error-mcp-method-not-found = विधि नहीं मिली: { $method }
error-mcp-invalid-params = अमान्य पैरामीटर: { $reason }
error-mcp-internal-error = MCP सर्वर आंतरिक त्रुटि: { $reason }
error-mcp-not-initialized = MCP सर्वर प्रारंभ नहीं किया गया है
error-mcp-tool-not-found = उपकरण नहीं मिला: { $tool }
error-mcp-tool-execution-failed = उपकरण "{ $tool }" का निष्पादन विफल: { $reason }
error-mcp-transport-error = MCP परिवहन त्रुटि: { $reason }
error-mcp-shutdown-failed = MCP सर्वर शटडाउन विफल: { $reason }

# MCP जानकारी
mcp-server-started = MCP सर्वर शुरू हुआ ({ $transport } परिवहन)
mcp-server-stopped = MCP सर्वर बंद हो गया
mcp-client-initialized = MCP क्लाइंट प्रारंभ हुआ: { $client_name }
mcp-tool-called = उपकरण कॉल किया गया: { $tool }
mcp-tool-completed = उपकरण "{ $tool }" { $duration }ms में पूरा हुआ
mcp-request-received = अनुरोध प्राप्त हुआ: { $method }
mcp-response-sent = प्रतिक्रिया भेजी गई: { $method }
mcp-transport-ready = MCP परिवहन तैयार: { $transport }

# Graph errors
error-graph-entity-not-found = ग्राफ़ इकाई नहीं मिली: { $id }
error-graph-relationship-failed = संबंध जोड़ने में विफल: { $reason }
error-graph-build-failed = ज्ञान ग्राफ़ बनाने में विफल: { $reason }
error-graph-update-failed = ज्ञान ग्राफ़ अपडेट करने में विफल: { $reason }
error-graph-load-failed = { $path } से ज्ञान ग्राफ़ लोड करने में विफल: { $reason }
error-graph-save-failed = { $path } में ज्ञान ग्राफ़ सहेजने में विफल: { $reason }
error-graph-max-entities-exceeded = ज्ञान ग्राफ़ ने अधिकतम इकाई सीमा पार कर दी: { $count } / { $max }

# Graph info
graph-built = ज्ञान ग्राफ़ बनाया गया: { $entities } इकाइयाँ और { $relationships } संबंध
graph-updated = ज्ञान ग्राफ़ अपडेट किया गया: { $added } जोड़ी गईं, { $removed } हटाई गईं
graph-entity-added = ज्ञान ग्राफ़ में इकाई जोड़ी गई: { $name } ({ $kind })
graph-entity-removed = ज्ञान ग्राफ़ से इकाई हटाई गई: { $name }
graph-persisted = ज्ञान ग्राफ़ { $path } में सहेजा गया
graph-loaded = ज्ञान ग्राफ़ { $path } से लोड किया गया ({ $entities } इकाइयाँ)
graph-query-executed = ग्राफ़ क्वेरी { $ms }ms में पूरी हुई, { $results } परिणाम

# प्लेटफ़ॉर्म API त्रुटियाँ
error-platform-api-request-failed = प्लेटफ़ॉर्म API अनुरोध विफल: { $reason }
error-platform-api-unauthorized = प्लेटफ़ॉर्म API प्रमाणीकरण विफल — channel_api_key जाँचें
error-platform-api-not-found = प्लेटफ़ॉर्म संसाधन नहीं मिला: { $resource }
error-platform-api-rate-limited = प्लेटफ़ॉर्म API दर सीमित — { $seconds }s बाद पुनः प्रयास करें
error-platform-api-server-error = प्लेटफ़ॉर्म सर्वर त्रुटि ({ $status }): { $message }
error-platform-trial-not-eligible = यह उपकरण परीक्षण के लिए पात्र नहीं है: { $reason }
error-platform-activation-failed = लाइसेंस सक्रियण विफल: { $reason }
error-platform-validation-failed = लाइसेंस सत्यापन विफल: { $reason }
error-platform-deactivation-failed = उपकरण निष्क्रियण विफल: { $reason }
error-platform-cache-read-failed = { $path } से लाइसेंस कैश पढ़ने में विफल: { $reason }
error-platform-cache-write-failed = { $path } में लाइसेंस कैश लिखने में विफल: { $reason }
error-platform-cache-decrypt-failed = लाइसेंस कैश डिक्रिप्ट करने में विफल (कुंजी बेमेल या डेटा भ्रष्टाचार)
error-platform-not-configured = प्लेटफ़ॉर्म एकीकरण कॉन्फ़िगर नहीं है — कॉन्फ़िगरेशन में platform_base_url सेट करें

# प्लेटफ़ॉर्म API जानकारी
platform-api-trial-activated = परीक्षण सक्रिय: { $tier } योजना, { $days } दिन
platform-api-license-activated = लाइसेंस सक्रिय: { $tier } योजना (सक्रियण { $activation_id })
platform-api-license-validated = लाइसेंस सत्यापित: { $tier } योजना, { $days } दिन शेष
platform-api-heartbeat-sent = हार्टबीट भेजा गया (सक्रियण { $activation_id })
platform-api-device-deactivated = उपकरण लाइसेंस से निष्क्रिय किया गया
platform-api-cache-updated = लाइसेंस कैश { $path } पर अपडेट किया गया
platform-api-offline-fallback = प्लेटफ़ॉर्म अनुपलब्ध, कैश किया हुआ लाइसेंस उपयोग कर रहे हैं ({ $days_ago } दिन पहले कैश किया गया)

# संदेश त्रुटियाँ
error-messaging-not-registered = संदेश क्लाइंट पंजीकृत नहीं है
error-messaging-registration-failed = संदेश पंजीकरण विफल: { $reason }
error-messaging-send-failed = संदेश भेजने में विफल: { $reason }
error-messaging-poll-failed = संदेश पोल करने में विफल: { $reason }
error-messaging-ack-failed = संदेश { $message_id } की पावती में विफल: { $reason }
error-messaging-disabled = इस लाइसेंस के लिए संदेश सेवा अक्षम है

# संदेश जानकारी
messaging-registered = उपकरण { $device_id } के लिए संदेश सेवा पंजीकृत
messaging-unregistered = संदेश सेवा अपंजीकृत
messaging-message-received = संदेश प्राप्त: { $subject } (प्रकार: { $message_type })
messaging-message-sent = संदेश भेजा गया (id: { $message_id })
messaging-poll-completed = संदेश पोल पूर्ण: { $count } नए संदेश

# Provider credential descriptions
credential-xai-api-key = Grok के लिए xAI API कुंजी (XAI_API_KEY)
credential-openai-api-key = OpenAI API कुंजी (OPENAI_API_KEY)
credential-google-api-key = Gemini के लिए Google API कुंजी (GOOGLE_API_KEY)
credential-gh-auth = gh CLI के माध्यम से GitHub प्रमाणीकरण (gh auth login)

# Built-in category names
category-SoftwareDevelopment = सॉफ्टवेयर विकास
category-LinuxDevelopment = Linux विकास
category-macOSDevelopment = macOS विकास
category-PythonDevelopment = Python विकास
category-AIFrameworks = AI और ML फ्रेमवर्क
category-GraphQL = GraphQL फ्रेमवर्क
category-DataScience = डेटा साइंस और एनालिटिक्स
category-Legal = कानूनी / पैरालीगल
category-Music = संगीत उत्पादन
category-PhysicalSystems = भौतिक प्रणालियाँ और घटनाएँ
category-BacteriaScience = जीवाणु विज्ञान और सूक्ष्म जीव विज्ञान
category-NursingScience = नर्सिंग विज्ञान और नैदानिक अभ्यास
category-ElectronDevelopment = Electron डेस्कटॉप विकास
category-GameDevelopment = गेम विकास
category-3DModeling = 3D मॉडलिंग और डिजिटल कंटेंट निर्माण
category-Custom = कस्टम टेम्पलेट

# Built-in category descriptions
category-SoftwareDevelopment-desc = एप्लिकेशन, API, डेटाबेस और स्क्रिप्ट बनाने के लिए टेम्पलेट
category-LinuxDevelopment-desc = Linux सिस्टम प्रशासन, शेल स्क्रिप्टिंग और सर्वर विकास के लिए टेम्पलेट
category-macOSDevelopment-desc = macOS एप्लिकेशन, Swift/Objective-C विकास और Apple फ्रेमवर्क के लिए टेम्पलेट
category-PythonDevelopment-desc = Python एप्लिकेशन, स्क्रिप्ट, वेब फ्रेमवर्क और स्वचालन के लिए टेम्पलेट
category-AIFrameworks-desc = AI एजेंट, LLM ऑर्केस्ट्रेशन, चैटबॉट और ML एप्लिकेशन के लिए टेम्पलेट
category-GraphQL-desc = GraphQL सर्वर, क्लाइंट और API विकास के लिए टेम्पलेट
category-DataScience-desc = डेटा साइंस जीवनचक्र के लिए टेम्पलेट: गणित, डेटा इंजीनियरिंग, ML, डीप लर्निंग, MLOps
category-Legal-desc = कानूनी दस्तावेज़ प्रसंस्करण, शोध और केस प्रबंधन के लिए टेम्पलेट
category-Music-desc = DAW, प्लगइन विकास, मॉड्यूलर सिंथेसिस और हार्डवेयर एकीकरण के लिए टेम्पलेट
category-PhysicalSystems-desc = औद्योगिक भौतिकी, प्रक्रिया निगरानी, नियंत्रण प्रणाली और भविष्य कथन विश्लेषिकी के लिए टेम्पलेट
category-BacteriaScience-desc = सूक्ष्म जीव विज्ञान, जीनोमिक्स, मेटाजीनोमिक्स, रोगाणुरोधी प्रतिरोध और निदान के लिए टेम्पलेट
category-NursingScience-desc = नर्सिंग शिक्षा, नैदानिक अभ्यास, रोगी देखभाल और स्वास्थ्य विश्लेषिकी के लिए टेम्पलेट
category-ElectronDevelopment-desc = Electron और आधुनिक टूलिंग के साथ क्रॉस-प्लेटफॉर्म डेस्कटॉप एप्लिकेशन के लिए टेम्पलेट
category-GameDevelopment-desc = गेम इंजन, फ्रेमवर्क और इंटरैक्टिव मनोरंजन विकास के लिए टेम्पलेट
category-3DModeling-desc = 3D मॉडलिंग, VFX, एनिमेशन और डिजिटल कंटेंट निर्माण टूल के लिए टेम्पलेट
category-Custom-desc = उपयोगकर्ता द्वारा बनाए गए कस्टम टेम्पलेट

# Provider status
provider-not-installed = प्रदाता { $provider } को { $binary } की आवश्यकता है जो इंस्टॉल नहीं है
provider-binary-found = { $binary } { $path } पर मिला
provider-test-timeout = कनेक्शन टेस्ट { $seconds } सेकंड के बाद समय समाप्त हो गया
provider-test-failed = प्रदाता परीक्षण विफल: { $error }
provider-env-saved = { $provider } के लिए { $env_var } सहेजा गया

# Worktree git integration messages
worktree-git-not-found = git executable not found: { $detail }
worktree-git-check-failed = git is not functional in this environment
worktree-not-git-repo = Path is not a git repository: { $path }
worktree-not-found = No active worktree found for this worker
worktree-dir-create-failed = Failed to create worktree directory at { $path }: { $detail }
worktree-merge-exec-failed = Failed to spawn git merge process: { $detail }
worktree-gitignore-read-failed = Failed to read .gitignore: { $detail }
worktree-gitignore-write-failed = Failed to write .gitignore: { $detail }
worktree-gitignore-create-failed = Failed to create .gitignore: { $detail }
worktree-git-exec-failed = Failed to spawn git command ({ $cmd }): { $detail }
worktree-git-cmd-failed = git command failed ({ $cmd }): { $detail }

# Autonomous Sprint Completion (WAVE 3)
error-orchestration-quorum-not-met = प्रबंधक कोरम पूरा नहीं हुआ: { $received } में से { $expected } प्रस्ताव प्राप्त हुए (न्यूनतम: { $min_quorum })
orchestration-file-conflict-serialized = कार्य { $later_task } को { $earlier_task } के बाद क्रमबद्ध किया गया — दोनों फ़ाइल '{ $file }' का दावा करते हैं
orchestration-build-check-failed = कर्मचारी { $worker_id } के लिए प्रति-कर्मचारी { $project_type } निर्माण जांच विफल
orchestration-build-check-passed = कर्मचारी { $worker_id } के लिए प्रति-कर्मचारी { $project_type } निर्माण जांच पास
