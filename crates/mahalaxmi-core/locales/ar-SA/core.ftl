# الأخطاء
error-config-file-not-found = لم يتم العثور على ملف الإعدادات في { $path }
error-config-parse-failed = فشل تحليل الإعدادات: { $reason }
error-config-validation-failed = فشل التحقق من صحة الإعدادات: { $details }
error-locale-not-supported = اللغة "{ $locale }" غير مدعومة
error-log-init-failed = فشل تهيئة السجل: { $reason }
error-log-dir-create-failed = فشل إنشاء مجلد السجل في { $path }: { $reason }
error-app-launch-failed = فشل في تشغيل التطبيق: { $reason }

# التحقق
validation-invalid-log-level = مستوى سجل غير صالح "{ $level }". المستويات الصالحة: { $valid }
validation-workers-out-of-range = يجب أن يكون max_concurrent_workers بين { $min } و{ $max }، القيمة المُدخلة: { $value }
validation-manager-timeout-too-low = يجب أن يكون manager_timeout_seconds على الأقل { $min }، القيمة المُدخلة: { $value }
validation-worker-timeout-too-low = يجب أن يكون worker_timeout_seconds على الأقل { $min }، القيمة المُدخلة: { $value }
validation-offline-grace-too-low = يجب أن يكون offline_grace_days على الأقل { $min }، القيمة المُدخلة: { $value }
validation-invalid-consensus-strategy = استراتيجية إجماع غير صالحة "{ $value }". الاستراتيجيات الصالحة: { $valid }
validation-invalid-data-directory = data_directory يحتوي على أحرف غير صالحة
validation-empty-default-provider = providers.default_provider يجب ألا يكون فارغاً
validation-invalid-theme = سمة غير صالحة "{ $value }". السمات الصالحة: { $valid }
validation-font-size-out-of-range = يجب أن يكون terminal_font_size بين { $min } و{ $max }، القيمة المُدخلة: { $value }
validation-invalid-max-batch-retries = u064au062cu0628 u0623u0646 u064au0643u0648u0646 max_batch_retries >= 1u060c u0627u0644u0642u064au0645u0629 u0627u0644u0645u064fu062fu062eu0644u0629: { $value }
validation-invalid-max-total-batches = u064au062cu0628 u0623u0646 u064au0643u0648u0646 max_total_batches >= 2u060c u0627u0644u0642u064au0645u0629 u0627u0644u0645u064fu062fu062eu0644u0629: { $value }
validation-invalid-stall-detection-threshold = u064au062cu0628 u0623u0646 u064au0643u0648u0646 stall_detection_threshold >= 2u060c u0627u0644u0642u064au0645u0629 u0627u0644u0645u064fu062fu062eu0644u0629: { $value }

# الإعدادات
config-loaded-successfully = تم تحميل الإعدادات من { $path }
config-using-default = لم يتم العثور على ملف إعدادات، يتم استخدام القيم الافتراضية
config-env-override = تم تجاوز قيمة الإعدادات بواسطة متغير البيئة { $var }
config-env-override-invalid = متغير البيئة { $var } يحتوي على قيمة غير صالحة "{ $value }"، يتم الاحتفاظ بالقيمة المُعدّة
config-generated-successfully = تم إنشاء الإعدادات الافتراضية في { $path }
config-already-exists = ملف الإعدادات موجود بالفعل في { $path }

# السجل
logging-initialized = تم تهيئة السجل عند المستوى { $level }
logging-rust-log-override = تم اكتشاف متغير البيئة RUST_LOG، يتم تجاوز المستوى المُعدّ
logging-file-path = ملف السجل: { $path }
logging-dir-create-failed-fallback = فشل إنشاء مجلد السجل { $path }، التحول إلى التسجيل في وحدة التحكم فقط

# PTY
pty-open-failed = فشل فتح الطرفية الزائفة: { $reason }
pty-spawn-failed = فشل تشغيل "{ $program }" في PTY: { $reason }
pty-write-failed = فشل الكتابة إلى الطرفية { $terminal_id }: { $reason }
pty-read-failed = فشل القراءة من الطرفية { $terminal_id }: { $reason }
pty-resize-failed = فشل تغيير حجم الطرفية { $terminal_id } إلى { $rows }×{ $cols }: { $reason }
pty-wait-failed = فشل التحقق من حالة العملية للطرفية { $terminal_id }: { $reason }
pty-kill-failed = فشل إنهاء العملية في الطرفية { $terminal_id }: { $reason }

# التطبيق
app-starting = Mahalaxmi الإصدار { $version } قيد التشغيل
app-ready = Mahalaxmi جاهز
app-shutting-down = جارٍ إيقاف Mahalaxmi

# بيانات الاعتماد
credential-anthropic-api-key = مفتاح API من Anthropic لـ Claude Code
credential-generic-api-key = مفتاح API لـ { $provider }
credential-aws-iam-role = دور AWS IAM لـ { $provider }
credential-oauth-token = رمز OAuth لـ { $provider }

# المزود
error-provider-credentials-missing = بيانات اعتماد { $provider } مفقودة: متغير البيئة { $env_var } غير معيّن
error-provider-credentials-invalid = بيانات اعتماد { $provider } غير صالحة: { $reason }
error-provider-not-found = المزود "{ $provider_id }" غير موجود في السجل
error-provider-no-default = لم يتم تعيين مزود ذكاء اصطناعي افتراضي
error-provider-command-build-failed = فشل في بناء أمر { $provider }: { $reason }
provider-registered = تم تسجيل المزود "{ $provider }" بالمعرّف "{ $id }"
provider-set-default = تم تعيين المزود الافتراضي إلى "{ $provider }"
provider-credentials-valid = تم التحقق من بيانات اعتماد { $provider } بنجاح
provider-validating = جارٍ التحقق من بيانات اعتماد { $provider }
provider-list-header = مزودو الذكاء الاصطناعي المسجلون

# PTY (موسّع)
error-pty-open-failed = فشل في فتح PTY: { $reason }
error-pty-spawn-failed = فشل في تشغيل "{ $program }" في PTY: { $reason }
error-pty-write-failed = فشل في الكتابة إلى الطرفية { $terminal_id }: { $reason }
error-pty-read-failed = فشل في القراءة من الطرفية { $terminal_id }: { $reason }
error-pty-resize-failed = فشل في تغيير حجم الطرفية { $terminal_id } إلى { $rows }x{ $cols }: { $reason }
error-pty-kill-failed = فشل في إنهاء عملية الطرفية { $terminal_id }: { $reason }
error-pty-wait-failed = فشل في التحقق من حالة عملية الطرفية { $terminal_id }: { $reason }
error-pty-terminal-not-found = الطرفية { $terminal_id } غير موجودة
error-pty-max-concurrent-reached = تم الوصول إلى الحد الأقصى للطرفيات المتزامنة ({ $max })
pty-process-spawned = تم تشغيل العملية "{ $program }" في الطرفية { $terminal_id }
pty-process-exited = انتهت عملية الطرفية { $terminal_id } بالرمز { $exit_code }
pty-session-closed = تم إغلاق جلسة الطرفية { $terminal_id }
pty-resized = تم تغيير حجم الطرفية { $terminal_id } إلى { $rows }x{ $cols }
pty-reader-eof = وصل قارئ الطرفية { $terminal_id } إلى نهاية الدفق
pty-reader-error = خطأ في قارئ الطرفية { $terminal_id }: { $reason }

# أخطاء التنسيق
error-orchestration-invalid-transition = انتقال حالة غير صالح من { $from } إلى { $to }
error-orchestration-circular-dependency = تم اكتشاف تبعية دائرية: { $cycle }
error-orchestration-worker-not-found = العامل { $worker_id } غير موجود في قائمة الانتظار
error-orchestration-max-retries-exceeded = تجاوز العامل { $worker_id } الحد الأقصى لمحاولات الإعادة ({ $max_retries })
error-orchestration-no-proposals = لم يتم استلام مقترحات من المديرين
error-orchestration-plan-validation-failed = فشل التحقق من خطة التنفيذ: { $errors }
error-orchestration-consensus-failed = فشل محرك الإجماع: { $reason }
error-orchestration-queue-full = قائمة انتظار العمال ممتلئة (الحد الأقصى { $max })
error-orchestration-manager-timeout = انتهت مهلة المدير { $manager_id } بعد { $timeout } ثانية
error-orchestration-worker-timeout = انتهت مهلة العامل { $worker_id } بعد { $timeout } ثانية

# معلومات التنسيق
orchestration-cycle-started = بدأت دورة التنسيق { $cycle_id }
orchestration-state-changed = تغيرت الحالة: { $from } -> { $to }
orchestration-manager-completed = أكمل المدير { $manager_id } بـ { $task_count } مهام
orchestration-consensus-reached = تم التوصل إلى إجماع: { $agreed } موافق، { $dissenting } معارض
orchestration-plan-created = تم إنشاء خطة التنفيذ: { $phases } مراحل، { $workers } عمال
orchestration-worker-started = بدأ العامل { $worker_id }: { $task }
orchestration-worker-completed = أكمل العامل { $worker_id } في { $duration } مللي ثانية
orchestration-worker-failed = فشل العامل { $worker_id }: { $error }
orchestration-cycle-completed = اكتملت الدورة في { $duration } مللي ثانية (معدل النجاح: { $success_rate })
orchestration-worker-retrying = العامل { $worker_id } يعيد المحاولة (المحاولة { $attempt }/{ $max })

# أخطاء الكشف
error-detection-rule-compile-failed = فشل تجميع نمط قاعدة الكشف: { $reason }
error-detection-no-rules-loaded = لم يتم تحميل قواعد كشف
error-detection-invalid-pattern = نمط كشف غير صالح "{ $pattern }": { $reason }

# معلومات الكشف
detection-rule-matched = قاعدة الكشف "{ $rule }" تطابقت، الإجراء: { $action }
detection-rule-cooldown = قاعدة الكشف "{ $rule }" تم تثبيطها بفترة التهدئة ({ $remaining_ms } مللي ثانية متبقية)
detection-rules-loaded = تم تحميل { $count } قواعد كشف
detection-provider-rules-applied = تم تطبيق { $count } قواعد للمزود { $provider }
detection-error-pattern-detected = تم اكتشاف نمط خطأ: "{ $pattern }" (شوهد { $count } مرات)
detection-root-cause-hypothesis = فرضية السبب الجذري: { $category } (الثقة: { $confidence })
detection-recurring-error = خطأ متكرر: "{ $message }" (حدث { $count } مرات)
detection-action-executed = تم تنفيذ الإجراء { $action } للقاعدة "{ $rule }"
detection-cooldowns-reset = تم إعادة تعيين فترات التهدئة لـ { $rule_count } قواعد

# أخطاء القوالب
error-template-not-found = القالب { $template_id } غير موجود
error-template-category-not-found = فئة القالب { $category_id } غير موجودة
error-template-composition-failed = فشل تركيب القالب: { $reason }
error-template-include-not-found = ملف التضمين غير موجود: { $path }
error-template-circular-include = تم اكتشاف تضمين دائري (تم تجاوز العمق الأقصى { $depth })
error-template-placeholder-unresolved = عنصر نائب غير محلول: ${ $placeholder }
error-template-validation-failed = فشل التحقق من القالب مع { $count } أخطاء
error-template-activation-failed = فشل تفعيل القالب: { $reason }
error-template-catalog-load-failed = فشل تحميل كتالوج القوالب: { $path }
error-template-invalid-version = تنسيق إصدار القالب غير صالح: { $version }

# معلومات القوالب
template-catalog-loaded = تم تحميل كتالوج القوالب مع { $count } قالب
template-activated = تم تفعيل القالب { $template_id } بنجاح
template-composition-complete = اكتمل التركيب: { $included } مضمنة، { $overridden } مستبدلة
template-placeholders-resolved = تم حل { $count } عنصر نائب
template-validation-passed = نجح التحقق من القالب للمجال { $domain }
template-validation-warnings = اكتمل التحقق من القالب مع { $count } تحذيرات
template-include-resolved = تم حل التضمين: { $path }
template-provider-instructions-injected = تم حقن تعليمات المزود لـ { $provider }
template-project-config-loaded = تم تحميل إعدادات المشروع من { $path }
template-domain-validator-registered = تم تسجيل مدقق المجال: { $domain }

# أخطاء الترخيص
error-license-file-not-found = لم يتم العثور على ملف الترخيص في { $path }
error-license-file-invalid = ملف ترخيص غير صالح في { $path }: { $reason }
error-license-file-write-failed = فشل في كتابة ملف الترخيص إلى { $path }: { $reason }
error-license-signature-invalid = فشل التحقق من توقيع الترخيص
error-license-signature-decode-failed = فشل في فك تشفير توقيع الترخيص: { $reason }
error-license-serialization-failed = فشل في تسلسل بيانات الترخيص: { $reason }
error-license-signing-failed = فشل في توقيع الترخيص: { $reason }
error-license-feature-denied = الميزة '{ $feature }' غير متوفرة في خطة { $tier }
error-license-worker-limit = عدد العمال المطلوب { $requested } يتجاوز حد خطة { $tier } البالغ { $limit }
error-license-manager-limit = عدد المديرين المطلوب { $requested } يتجاوز حد خطة { $tier } البالغ { $limit }. فعّل اشتراكك من الإعدادات ← الترخيص للحصول على المزيد.
error-license-category-denied = الفئة '{ $category }' تتطلب خطة { $required_tier } (الحالية: { $tier })
error-license-fingerprint-hostname = فشل في تحديد اسم المضيف: { $reason }
error-license-fingerprint-username = فشل في تحديد اسم المستخدم: { $reason }

# حالة الترخيص
license-trial-active = ترخيص تجريبي نشط ({ $days } أيام متبقية)
license-trial-expiring-soon = الفترة التجريبية ستنتهي قريباً ({ $days } أيام متبقية)
license-trial-expiring-very-soon = الفترة التجريبية على وشك الانتهاء ({ $days } أيام متبقية)
license-trial-expired = انتهت صلاحية الترخيص التجريبي
license-expires-later = ينتهي الترخيص خلال { $days } أيام
license-expires-soon = الترخيص سينتهي قريباً ({ $days } أيام متبقية)
license-expires-very-soon = الترخيص على وشك الانتهاء ({ $days } أيام متبقية)
license-expires-today = ينتهي الترخيص اليوم
license-grace-period = انتهى الترخيص، فترة السماح نشطة ({ $days } أيام متبقية)
license-expired = انتهت صلاحية الترخيص

# دعم المنصة — مفاتيح الخطأ
error-platform-unsupported = المنصة غير مدعومة: { $platform }
error-platform-wsl-not-detected = لم يتم اكتشاف بيئة WSL
error-platform-wsl-path-invalid = مسار غير صالح لترجمة WSL: { $path }
error-platform-layout-no-space = الحاوية صغيرة جداً لتخطيط اللوحات ({ $width }x{ $height })
error-platform-layout-invalid-count = عدد لوحات غير صالح: { $count }
error-platform-hotkey-registration-failed = فشل تسجيل الاختصار (تعارض): { $shortcut }
error-platform-hotkey-parse-failed = فشل تحليل اختصار لوحة المفاتيح: { $shortcut }
error-platform-shutdown-timeout = انتهت مهلة إيقاف العملية { $pid } ({ $label })
error-platform-shutdown-failed = فشل إيقاف العملية { $pid } ({ $label })
error-platform-shell-not-found = لم يتم العثور على الصدفة الافتراضية

# دعم المنصة — مفاتيح المعلومات
platform-detected = تم اكتشاف المنصة: { $os } ({ $arch })
platform-wsl-detected = تم اكتشاف WSL: { $distro } (WSL{ $version })
platform-wsl-path-translated = تمت ترجمة المسار: { $from } → { $to }
platform-layout-calculated = تم حساب التخطيط: { $panels } لوحة في شبكة { $rows }x{ $cols }
platform-layout-optimized = تم تحسين التخطيط: { $utilization }% استخدام
platform-hotkey-registered = تم تسجيل الاختصار: { $command } → { $shortcut }
platform-hotkey-unregistered = تم إلغاء تسجيل الاختصار: { $command }
platform-shutdown-initiated = بدأ إيقاف { $count } عمليات
platform-shutdown-completed = اكتمل الإيقاف: { $count } عمليات في { $duration }ms
platform-shell-detected = تم اكتشاف الصدفة: { $shell } ({ $path })

# أخطاء الذاكرة
error-memory-not-found = إدخال الذاكرة غير موجود: { $id }
error-memory-duplicate = إدخال ذاكرة مكرر: { $id }
error-memory-persistence-failed = فشل في حفظ مخزن الذاكرة: { $reason }
error-memory-load-failed = فشل في تحميل مخزن الذاكرة: { $reason }
error-memory-invalid-confidence = درجة ثقة غير صالحة: { $value } (يجب أن تكون بين 0.0 و 1.0)
error-memory-store-full = مخزن الذاكرة ممتلئ (الحد الأقصى { $max } إدخالات)
error-memory-invalid-query = استعلام ذاكرة غير صالح: { $reason }
error-memory-serialization = فشل في تسلسل الذاكرة: { $reason }
error-memory-invalid-entry = إدخال ذاكرة غير صالح: { $reason }
error-memory-session-mismatch = عدم تطابق الجلسة: المتوقع { $expected }، الفعلي { $actual }

# معلومات الذاكرة
memory-store-created = تم إنشاء مخزن الذاكرة للجلسة { $session_id }
memory-entry-added = تمت إضافة إدخال ذاكرة: { $title } (النوع: { $memory_type })
memory-entry-updated = تم تحديث إدخال الذاكرة: { $id }
memory-entry-removed = تمت إزالة إدخال الذاكرة: { $id }
memory-store-cleared = تم مسح مخزن الذاكرة ({ $count } إدخالات تمت إزالتها)
memory-persisted = تم حفظ مخزن الذاكرة في { $path }
memory-loaded = تم تحميل مخزن الذاكرة من { $path } ({ $count } إدخالات)
memory-query-executed = استعلام الذاكرة أعاد { $count } نتائج
memory-injected = تم حقن { $count } ذكريات ({ $tokens } رموز)
memory-stats = إحصائيات الذاكرة: { $total } إدخالات، متوسط الثقة { $avg_confidence }

# أخطاء الفهرسة
error-indexing-parse-failed = فشل تحليل { $file }: { $reason }
error-indexing-file-read-failed = فشل قراءة الملف { $file }: { $reason }
error-indexing-unsupported-language = لغة غير مدعومة لامتداد الملف: { $extension }
error-indexing-extraction-failed = فشل استخراج الرموز من { $file }: { $reason }
error-indexing-graph-cycle-detected = تم اكتشاف دورة تبعية: { $files }
error-indexing-fingerprint-failed = فشل حساب البصمة لـ { $file }: { $reason }
error-indexing-build-failed = فشل بناء الفهرس: { $reason }
error-indexing-update-failed = فشل التحديث التراكمي: { $reason }

# معلومات الفهرسة
indexing-file-indexed = تم فهرسة الملف: { $file } ({ $language })
indexing-symbols-extracted = تم استخراج { $count } رمز من { $file }
indexing-graph-built = تم بناء رسم التبعية: { $files } ملفات، { $edges } حواف
indexing-ranking-computed = تم حساب الترتيب لـ { $symbols } رمز
indexing-repomap-generated = تم إنشاء خريطة المستودع: { $symbols } رمز، { $tokens } رمز مميز
indexing-index-built = تم بناء فهرس قاعدة الشفرة: { $files } ملفات، { $symbols } رمز
indexing-incremental-update = تحديث تراكمي: { $added } مضاف، { $modified } معدّل، { $removed } محذوف
indexing-language-registered = تم تسجيل اللغة: { $language }

# أخطاء السياق
error-context-budget-exceeded = تم تجاوز ميزانية رموز السياق: مستخدم { $used }، الميزانية { $budget }
error-context-invalid-allocations = يجب أن يكون مجموع تخصيصات الميزانية <= 1.0، الناتج { $sum }
error-context-build-failed = فشل بناء السياق للمهمة { $task_id }: { $reason }
error-context-invalid-format = تنسيق سياق غير صالح: { $format }

# معلومات السياق
context-budget-allocated = تم تخصيص ميزانية الرموز: { $total } رمز ({ $repo_map } خريطة المستودع، { $files } ملفات، { $memory } ذاكرة، { $task } مهمة)
context-files-scored = تم تقييم { $count } ملف للصلة (الأعلى: { $top_file })
context-chunks-created = تم إنشاء { $count } قطعة شفرة ({ $tokens } رمز)
context-assembled = تم تجميع السياق: { $sections } أقسام، { $tokens } رمز مستخدم من { $budget } ميزانية
context-injected = تم حقن السياق للعامل { $worker_id } ({ $tokens } رمز، { $files } ملفات)
context-skipped = تم تخطي إعداد السياق: { $reason }

# أخطاء MCP
error-mcp-parse-failed = فشل في تحليل رسالة JSON-RPC: { $reason }
error-mcp-invalid-request = طلب JSON-RPC غير صالح: { $reason }
error-mcp-method-not-found = الطريقة غير موجودة: { $method }
error-mcp-invalid-params = معلمات غير صالحة: { $reason }
error-mcp-internal-error = خطأ داخلي في خادم MCP: { $reason }
error-mcp-not-initialized = لم يتم تهيئة خادم MCP
error-mcp-tool-not-found = الأداة غير موجودة: { $tool }
error-mcp-tool-execution-failed = فشل تنفيذ الأداة "{ $tool }": { $reason }
error-mcp-transport-error = خطأ في نقل MCP: { $reason }
error-mcp-shutdown-failed = فشل إيقاف خادم MCP: { $reason }

# معلومات MCP
mcp-server-started = تم بدء خادم MCP (نقل { $transport })
mcp-server-stopped = تم إيقاف خادم MCP
mcp-client-initialized = تم تهيئة عميل MCP: { $client_name }
mcp-tool-called = تم استدعاء الأداة: { $tool }
mcp-tool-completed = اكتملت الأداة "{ $tool }" في { $duration } مللي ثانية
mcp-request-received = تم استلام الطلب: { $method }
mcp-response-sent = تم إرسال الرد: { $method }
mcp-transport-ready = نقل MCP جاهز: { $transport }

# Graph errors
error-graph-entity-not-found = لم يتم العثور على كيان الرسم البياني: { $id }
error-graph-relationship-failed = فشل في إضافة العلاقة: { $reason }
error-graph-build-failed = فشل في بناء رسم المعرفة البياني: { $reason }
error-graph-update-failed = فشل في تحديث رسم المعرفة البياني: { $reason }
error-graph-load-failed = فشل في تحميل رسم المعرفة البياني من { $path }: { $reason }
error-graph-save-failed = فشل في حفظ رسم المعرفة البياني في { $path }: { $reason }
error-graph-max-entities-exceeded = تجاوز رسم المعرفة البياني الحد الأقصى للكيانات: { $count } / { $max }

# Graph info
graph-built = تم بناء رسم المعرفة البياني: { $entities } كيان و { $relationships } علاقة
graph-updated = تم تحديث رسم المعرفة البياني: { $added } مضافة، { $removed } محذوفة
graph-entity-added = تمت إضافة كيان إلى رسم المعرفة البياني: { $name } ({ $kind })
graph-entity-removed = تمت إزالة كيان من رسم المعرفة البياني: { $name }
graph-persisted = تم حفظ رسم المعرفة البياني في { $path }
graph-loaded = تم تحميل رسم المعرفة البياني من { $path } ({ $entities } كيان)
graph-query-executed = تم تنفيذ استعلام الرسم البياني في { $ms }مللي ثانية، { $results } نتيجة

# أخطاء واجهة برمجة تطبيقات المنصة
error-platform-api-request-failed = فشل طلب واجهة برمجة تطبيقات المنصة: { $reason }
error-platform-api-unauthorized = فشل مصادقة واجهة برمجة تطبيقات المنصة — تحقق من channel_api_key
error-platform-api-not-found = مورد المنصة غير موجود: { $resource }
error-platform-api-rate-limited = تم تجاوز حد معدل واجهة برمجة تطبيقات المنصة — أعد المحاولة بعد { $seconds } ثانية
error-platform-api-server-error = خطأ في خادم المنصة ({ $status }): { $message }
error-platform-trial-not-eligible = هذا الجهاز غير مؤهل للتجربة: { $reason }
error-platform-activation-failed = فشل تفعيل الترخيص: { $reason }
error-platform-validation-failed = فشل التحقق من الترخيص: { $reason }
error-platform-deactivation-failed = فشل إلغاء تفعيل الجهاز: { $reason }
error-platform-cache-read-failed = فشل في قراءة ذاكرة التخزين المؤقت للترخيص من { $path }: { $reason }
error-platform-cache-write-failed = فشل في كتابة ذاكرة التخزين المؤقت للترخيص إلى { $path }: { $reason }
error-platform-cache-decrypt-failed = فشل في فك تشفير ذاكرة التخزين المؤقت للترخيص (عدم تطابق المفتاح أو تلف البيانات)
error-platform-not-configured = تكامل المنصة غير مُعدّ — قم بتعيين platform_base_url في الإعدادات

# معلومات واجهة برمجة تطبيقات المنصة
platform-api-trial-activated = تم تفعيل التجربة: خطة { $tier }، { $days } أيام
platform-api-license-activated = تم تفعيل الترخيص: خطة { $tier } (التفعيل { $activation_id })
platform-api-license-validated = تم التحقق من الترخيص: خطة { $tier }، متبقي { $days } أيام
platform-api-heartbeat-sent = تم إرسال نبضة القلب (التفعيل { $activation_id })
platform-api-device-deactivated = تم إلغاء تفعيل الجهاز من الترخيص
platform-api-cache-updated = تم تحديث ذاكرة التخزين المؤقت للترخيص في { $path }
platform-api-offline-fallback = المنصة غير قابلة للوصول، يتم استخدام الترخيص المخزن مؤقتاً (تم التخزين قبل { $days_ago } أيام)

# أخطاء المراسلة
error-messaging-not-registered = عميل المراسلة غير مسجل
error-messaging-registration-failed = فشل تسجيل المراسلة: { $reason }
error-messaging-send-failed = فشل في إرسال الرسالة: { $reason }
error-messaging-poll-failed = فشل في استطلاع الرسائل: { $reason }
error-messaging-ack-failed = فشل في تأكيد استلام الرسالة { $message_id }: { $reason }
error-messaging-disabled = المراسلة معطلة لهذا الترخيص

# معلومات المراسلة
messaging-registered = تم تسجيل المراسلة للجهاز { $device_id }
messaging-unregistered = تم إلغاء تسجيل المراسلة
messaging-message-received = تم استلام رسالة: { $subject } (النوع: { $message_type })
messaging-message-sent = تم إرسال الرسالة (المعرّف: { $message_id })
messaging-poll-completed = اكتمل استطلاع الرسائل: { $count } رسائل جديدة

# Provider credential descriptions
credential-xai-api-key = مفتاح API لـ xAI لـ Grok (XAI_API_KEY)
credential-openai-api-key = مفتاح API لـ OpenAI (OPENAI_API_KEY)
credential-google-api-key = مفتاح API لـ Google لـ Gemini (GOOGLE_API_KEY)
credential-gh-auth = مصادقة GitHub عبر gh CLI (gh auth login)

# Built-in category names
category-SoftwareDevelopment = تطوير البرمجيات
category-LinuxDevelopment = تطوير Linux
category-macOSDevelopment = تطوير macOS
category-PythonDevelopment = تطوير Python
category-AIFrameworks = أطر عمل AI و ML
category-GraphQL = أطر عمل GraphQL
category-DataScience = علم البيانات والتحليلات
category-Legal = القانون / المساعدة القانونية
category-Music = إنتاج الموسيقى
category-PhysicalSystems = الأنظمة الفيزيائية والظواهر
category-BacteriaScience = علم البكتيريا والأحياء الدقيقة
category-NursingScience = علوم التمريض والممارسة السريرية
category-ElectronDevelopment = تطوير سطح المكتب بـ Electron
category-GameDevelopment = تطوير الألعاب
category-3DModeling = النمذجة ثلاثية الأبعاد وإنشاء المحتوى الرقمي
category-Custom = قوالب مخصصة

# Built-in category descriptions
category-SoftwareDevelopment-desc = قوالب لإنشاء التطبيقات وواجهات API وقواعد البيانات والنصوص البرمجية
category-LinuxDevelopment-desc = قوالب لإدارة أنظمة Linux وبرمجة Shell وتطوير الخوادم
category-macOSDevelopment-desc = قوالب لتطبيقات macOS وتطوير Swift/Objective-C وأطر عمل Apple
category-PythonDevelopment-desc = قوالب لتطبيقات Python والنصوص البرمجية وأطر عمل الويب والأتمتة
category-AIFrameworks-desc = قوالب لوكلاء AI وتنسيق LLM وروبوتات الدردشة وتطبيقات ML
category-GraphQL-desc = قوالب لخوادم GraphQL والعملاء وتطوير API
category-DataScience-desc = قوالب لدورة حياة علم البيانات: الرياضيات، هندسة البيانات، ML، التعلم العميق، MLOps
category-Legal-desc = قوالب لمعالجة المستندات القانونية والبحث وإدارة القضايا
category-Music-desc = قوالب لمحطات العمل الصوتية الرقمية وتطوير الإضافات والتوليف المعياري وتكامل الأجهزة
category-PhysicalSystems-desc = قوالب للفيزياء الصناعية ومراقبة العمليات وأنظمة التحكم والتحليلات التنبؤية
category-BacteriaScience-desc = قوالب لعلم الأحياء الدقيقة والجينوميات والميتاجينوميات ومقاومة مضادات الميكروبات والتشخيص
category-NursingScience-desc = قوالب لتعليم التمريض والممارسة السريرية ورعاية المرضى وتحليلات الرعاية الصحية
category-ElectronDevelopment-desc = قوالب لتطبيقات سطح المكتب متعددة المنصات باستخدام Electron والأدوات الحديثة
category-GameDevelopment-desc = قوالب لمحركات الألعاب والأطر وتطوير الترفيه التفاعلي
category-3DModeling-desc = قوالب للنمذجة ثلاثية الأبعاد والمؤثرات البصرية والرسوم المتحركة وأدوات إنشاء المحتوى الرقمي
category-Custom-desc = قوالب مخصصة أنشأها المستخدم

# Provider status
provider-not-installed = المزود { $provider } يتطلب { $binary } الذي لم يتم تثبيته
provider-binary-found = تم العثور على { $binary } في { $path }
provider-test-timeout = انتهت مهلة اختبار الاتصال بعد { $seconds } ثانية
provider-test-failed = فشل اختبار المزود: { $error }
provider-env-saved = تم حفظ { $env_var } لـ { $provider }

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
error-orchestration-quorum-not-met = لم يتم تحقيق النصاب القانوني للمدراء: { $received } من { $expected } مقترح تم استلامه (الحد الأدنى: { $min_quorum })
orchestration-file-conflict-serialized = المهمة { $later_task } تسلسلت بعد { $earlier_task } — كلاهما يطالب بالملف '{ $file }'
orchestration-build-check-failed = فشل فحص بناء { $project_type } للعامل { $worker_id }
orchestration-build-check-passed = نجح فحص بناء { $project_type } للعامل { $worker_id }
