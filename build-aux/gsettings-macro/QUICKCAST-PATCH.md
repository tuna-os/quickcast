This is the MPL-2.0 source of SeaDve/gsettings-macro 0.2.3, obtained from its
published crates.io archive. The only code/dependency change is updating
quick-xml from 0.39 to 0.41 to address the XML parser advisories reported by the
release audit. The macro parses this repository's GSettings schema at build time.
The original LICENSE and README are retained. Remove this patch when an upstream
release supports the fixed parser.
