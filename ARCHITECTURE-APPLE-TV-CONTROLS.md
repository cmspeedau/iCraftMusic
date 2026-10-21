# Architecture / Design — Living-Room Appliance Controls

**Status: Proposed / Unknown at integration boundary**  
**Scope:** client UI, remote-control behavior, and the Kodi handoff surface for the
Kodi appliance proposal. This document does not select an OS, CPU, remote protocol,
Kodi add-on/API, IPC mechanism, boot model, or hardware target.

The design keeps the iCraftMusic conventions that are already explicit in the project
files: Rust-only application code; native UI rather than a JavaScript runtime or web
view; local-first behavior; typed, actionable states; persisted settings; bounded
background work; and no claim of support until behavior is verified on the running
target. Kodi remains an external playback/browser owner until the integration boundary
is selected.

## 1. Product shape and ownership

The client is a focused appliance control surface, not a second media player. It owns
configuration, drive/sync state, diagnostics, update state, and the user-visible
handoff to Kodi. Kodi owns media browsing and playback once the handoff succeeds.

```text
Remote input
    │
    ▼
Input translator ──► Focus/navigation state ──► Client screens and overlays
    │                                      └──► Command coordinator
    ▼                                               │
Physical key / button semantics                 Kodi adapter (boundary TBD)
                                                    │
                                                    ▼
                                               Kodi browsing/playback
```

The command coordinator is the single owner of operations that can affect appliance
state. It prevents a focused button, a menu item, and a remote shortcut from issuing
separate competing actions. Long-running work (sync, scan, update download, Kodi
refresh) reports progress through a state channel and never blocks the UI frame loop.

### Ownership rules

| Surface or operation | Client owns | Kodi owns | Handoff rule |
| --- | --- | --- | --- |
| Appliance home, storage, sync, diagnostics, settings, update UI | Yes | No | No Kodi command is sent merely because focus changes. |
| Media browsing and playback after a successful handoff | No | Yes | Client sends only the selected, confirmed handoff command through the unchosen adapter boundary. |
| Managed media root and sync manifest | Yes | No | Kodi sees a stable exported root; it does not become the sync database owner. |
| Kodi refresh/invalidation | Requests and reports result | Performs refresh if supported | Refresh is explicit after a successful sync; unsupported/unconfirmed behavior is an error state, not a silent success. |
| Remote input while Kodi owns playback | Only reserved/system actions | Yes, for playback/navigation | Client must not steal ordinary playback buttons while Kodi is active. |
| Full-screen update flow | Yes | Temporarily obscures Kodi | Update controls remain client-owned; playback is paused or left running only according to a verified policy. |

A connection or process boundary must not be implied by this table. The adapter may
later be an add-on, local service, supported Kodi mechanism, or filesystem-only
contract; the choice remains **Unknown**, as recorded in this document.

### Local-LAN coordinator and device model

The **coordinator works over the local LAN**. It is the trusted control-plane owner for
paired appliance devices; it is not a cloud relay and does not require an Internet
service for normal control, update transfer, staging, reboot coordination, or health
reporting. “Local” means reachable on the same user-controlled LAN according to the
target's verified network behavior. LAN connectivity is intentionally an open design
decision: **no LAN port number, transport, service-discovery mechanism, or wire
protocol is specified or implied here**. Those values remain **Unknown/undecided**
until a target implementation is selected, reviewed, and tested.

The roles are deliberately separate:

| Role | Responsibility | Authority boundary |
| --- | --- | --- |
| **Coordinator** | Discovers eligible devices, presents pairing, maintains device identity and capabilities, queues idempotent commands, transfers update artifacts, tracks staging/reboot state, and reports results. | May request an operation on a paired device; does not claim success until the device acknowledges the corresponding durable state or health result. |
| **Device** | Runs the appliance agent, advertises eligibility, proves its identity during pairing, validates and executes authorized requests, stages images locally, owns boot/recovery state, and reports health. | Owns local safety decisions, signature/integrity/compatibility checks, activation, rollback, and the final reboot/health outcome. |

A coordinator may manage more than one device, but each operation is scoped to an
explicit device identity. The device remains safe if the coordinator disappears: a
transfer can pause or resume, staging can complete or fail locally, and an activation
that has entered its non-cancelable critical section must finish or recover according
to the device boot contract. The coordinator must not be treated as the device's boot
manager or as the sole source of truth for update state.

#### Discovery and pairing

1. The coordinator listens for eligible device advertisements on the local LAN using
   the mechanism selected for the target. Discovery is informational only; an
   advertisement does not grant control. The advertisement should expose only the
   minimum non-secret identity, reachability, and capability information needed to
   show a pairing candidate.
2. The user selects a visible device and starts an explicit pairing ceremony on the
   coordinator and, where supported, confirms on the device. Pairing binds a stable
   device identity to a coordinator-held credential or key reference; it is not
   approval based only on hostname, IP address, or an unauthenticated discovery
   response.
3. The ceremony displays a short-lived comparison value or equivalent human-verifiable
   confirmation when the chosen mechanism supports it. Rejected, expired, repeated,
   or ambiguous attempts fail closed and leave no active trust relationship.
4. The coordinator stores paired-device metadata in the existing persisted-settings
   and diagnostics model without storing exportable private key material in logs.
   Unpairing revokes the local relationship and stops new operations; the device must
   reject the revoked credential when the selected revocation design supports device
   enforcement.

Discovery must tolerate address changes, duplicate/stale advertisements, sleep, and
LAN segmentation. A previously paired device is recognized by its authenticated stable
identity, not by a remembered address alone. The UI distinguishes **Discovered**,
**Pairing required**, **Paired but unreachable**, **Authenticated**, and **Failed**.

#### Authenticated, encrypted communication

After pairing, coordinator/device traffic uses mutually authenticated, encrypted
communication with confidentiality, integrity, replay protection, and explicit
request identity. The concrete cryptographic library, session protocol, certificate
or key format, LAN transport, LAN port assignment, and wire protocol are all
**Unknown/undecided**. This document specifies **no fixed port number and no fixed
transport or wire protocol**; discovery metadata is not trusted as proof of identity,
and credentials never appear in diagnostics or user-visible errors.

Requests carry a device identity, operation identifier, capability/version context,
and enough durable state for idempotent retry. Responses identify the same operation
and distinguish accepted, in-progress, completed, rejected, expired, and unknown
outcome. Sensitive commands—pairing changes, image staging, activation, reboot, and
recovery—require the authenticated channel plus device-side authorization and local
preconditions. The device validates artifact identity and policy independently; an
authenticated coordinator cannot bypass verification or force an unsafe reboot.

#### Update transfer, staging, and reboot coordination

The coordinator may obtain an update artifact from a configured source and transfer
it to a paired device over the LAN. The source may be local or remote according to
policy, but the device must receive enough metadata to verify the artifact, including
its identity, compatibility, size when known, integrity information, and signature or
trust-chain data. The device—not the coordinator—performs the authoritative trust,
integrity, compatibility, and complete-payload checks before staging.

Transfer is resumable and bounded: chunks or equivalent units are acknowledged only
when durably accepted, retries are idempotent, and a disconnected coordinator does not
turn a partial artifact into an available update. The device stages to an inactive
slot or approved recovery-safe area while its current system remains usable. The
coordinator shows measured progress when available, otherwise the device phase and an
indeterminate state; it never infers staging or activation from bytes sent.

The coordinator requests **Ready to activate** only after the device reports complete
verification and staging. The user confirmation is made against the selected device
and candidate version. The device then durably records activation intent, owns the
non-cancelable reboot critical section, and reports **Preparing**, **Activating**,
**Health check**, **Complete**, or **Rolled back** after boot. Lost LAN connectivity during
that section must not leave activation dependent on the coordinator. Reconnection
reconciles journal state by operation/device identity rather than replaying a reboot.

#### Offline and no-cloud behavior

Normal paired-device control, discovery on the available LAN, diagnostics, local
media/storage management, transfer of an already available artifact, staging, and
reboot coordination work without a cloud account or cloud control path. Internet
access may be required by a selected update source or release check, but its absence
leaves the appliance usable and reports **Unknown**, **Unavailable**, or **Pending**
where appropriate; it must not be presented as up to date. A coordinator can stage an
artifact from local storage or another approved LAN source when policy and verification
permit it.

If the coordinator is offline, the device continues its local safety and boot/recovery
logic and retains durable operation state. If the device is offline, the coordinator
shows **Paired but unreachable**, queues only explicitly retryable/idempotent work, and
never claims delivery, staging, reboot, or health success. Cloud telemetry, remote
command relays, mandatory cloud pairing, and cloud-only recovery are out of scope for
this design.

## 2. Screen model and focus navigation

The main client uses a small, stable hierarchy rather than a desktop-style source
list. Recommended top-level destinations are:

- **Home** — current storage/Kodi readiness and the primary media handoff.
- **Media** — the entry point into Kodi browsing, not a duplicate library.
- **Sync** — dry run, plan, progress, completion, and recovery.
- **Settings** — storage, network/update policy, Kodi status, and accessibility.
- **Diagnostics** — build, platform, drive, managed roots, sync journal, Kodi boundary,
  and update state.

Each screen has one logical focus scope. Focus is visible at all times and is never
communicated by color alone. The initial focus is deterministic: the screen's primary
action when available, otherwise its first enabled control. Focus memory is per screen
only if it can be restored safely; a missing or disabled control falls back to the
screen's primary action.

### Standard remote map

| Remote action | Client behavior |
| --- | --- |
| Up / Down | Move to the nearest enabled control in the same logical column/row. Do not wrap by default. |
| Left / Right | Move within the current row; at a boundary, move to the next logical region only where the layout defines one. |
| Select / OK | Activate the focused control; on a menu row, open its submenu or perform its stated action. |
| Menu / Back | Close the topmost menu, overlay, dialog, or submenu; otherwise return to the previous screen. Never silently discard a pending destructive confirmation. |
| Play/Pause | Forward to Kodi only while Kodi owns playback; in a client-owned operation, pause/resume only if the client has a verified operation for it. |
| Long press Menu | Reserved for a platform/system action; do not assign an appliance action until the target OS/remote is confirmed. |
| Home / TV button | Reserved for the platform. The client must not intercept it. |

No hidden gesture is required to reach a critical action. A focused control exposes a
short label and state; a destructive action is a second-step confirmation with an
explicit Cancel choice. Menu/Back is idempotent: repeated presses close one layer at
a time and never terminate an in-progress write without a confirmation state.

### Menus and dialogs

Use familiar living-room TV interaction patterns for short contextual menus and full
screens for settings or multi-step work. The visual language may be Apple-like in its
clarity and restraint, but this proposal does not claim Apple branding, product identity,
or an exact copy of any Apple interface. A menu:

1. opens from the focused control and places focus on its first enabled item;
2. traps focus until Select chooses, Menu/Back cancels, or the menu closes itself after
   a completed action;
3. states the current choice with text plus a non-color indicator;
4. does not perform work merely by highlighting an item; and
5. returns focus to the invoking control after dismissal.

Dialogs are reserved for confirmation, authentication if later required, errors, and
progress that cannot be represented on the current screen. Dialogs are modal to the
client and must not leave an underlying control accidentally activatable.

## 3. Playback and Kodi handoff

The client does not reproduce iCraftMusic's desktop transport inside the appliance
shell. The playback bar, queue, seek, volume, decoder, and missing-file behavior in
`README.md`/`ROADMAP.md` describe the existing desktop player; this appliance design
must not claim those controls until Kodi ownership and the remote command path are
verified.

### Handoff sequence

1. Client shows the media destination and the current storage/Kodi readiness.
2. Select enters the handoff state: **Preparing**, **Ready**, **Unavailable**, or
   **Failed**. It does not claim that Kodi is ready from process presence alone.
3. If the selected adapter supports the action, the client sends one idempotent
   handoff command and waits for an acknowledgement or a bounded timeout.
4. On success, ownership changes to Kodi and the client yields ordinary navigation and
   playback buttons. A small client-owned status affordance may remain available only
   if the chosen boundary supports it.
5. On failure or timeout, the client keeps focus on the retry/diagnostics choices and
   states what was confirmed (for example, storage ready but Kodi refresh unconfirmed).
6. Returning to the client is an explicit Back/Menu boundary or platform behavior,
   not an assumed process kill or restart.

The handoff must be idempotent: retrying after an unknown acknowledgement must not
create duplicate sync work or an ambiguous second playback request. The exact
acknowledgement, timeout, resume behavior, and whether Kodi is paused/foregrounded
remain decisions for the selected integration.

## 4. Full-screen appliance update experience

The update experience is a client-owned, full-screen appliance flow: calm, sparse,
remote-first, and readable from a distance. It is inspired by familiar TV appliance
patterns rather than copied from a named product. It must take over the display before
activation and remain the only actionable surface through reboot preparation. If the
platform cannot safely foreground the client above Kodi, the coordinator must enter the
full-screen client page at the next verified boundary; it must not assume a window
manager, Kodi embedding, or always-on-top facility.

**Update scope clarification:** an iCraftMusic update on this appliance includes the
Buildroot environment and therefore updates the whole appliance system (including the
client and its runtime), not an app self-update. The client must not describe this as
replacing only its own binary, and must not reuse the desktop AppImage installation
model. The exact image/container format and boot integration remain target decisions.

### Boot, firmware, and partitioning for safe Buildroot updates

A special partition layout is not required merely because the appliance uses Buildroot,
but safe rebooting updates do require a boot contract that can select a complete,
verified system and recover if it does not become healthy. Keep the layers distinct:

- **Bootloader and firmware** are the platform-dependent path that initializes hardware,
  locates storage, verifies or authenticates images where supported, selects the boot
  target, and provides recovery/fallback behavior. Firmware may also impose fixed
  boot partitions, signing rules, or an update mechanism that must not be overwritten
  by the Linux update.
- **Linux partitioning** is the storage arrangement for the kernel, device tree or
  firmware handoff files, root filesystem images, recovery assets, and persistent data.
  It supports the boot contract; it does not replace the bootloader/firmware contract.

The recommended appliance arrangement is **A/B root filesystem slots** (for example,
rootfs A and rootfs B), with application/user data, media, configuration that must
survive an update, the update journal, and diagnostics in a separate persistent data
area. The device runs one slot while the complete signed image is written and verified
in the inactive slot. Only then does it atomically request that the bootloader select
the new slot on the next reboot. This keeps weekly validated stable releases aligned
with the LAN coordinator's staged, user-confirmed, rebooting flow: a coordinator may
transfer and stage an artifact, but the device owns activation and health.

Boot selection must be durable and failure-aware. The bootloader environment (or the
platform's equivalent boot metadata) should record the active slot, pending slot,
boot-attempt limit, and a success/health confirmation. A pending slot that does not
confirm health within the defined window is marked failed and the bootloader falls
back to the previous known-good slot. A recovery image or documented recovery path
should remain available for the case where neither rootfs slot boots; recovery must
not depend on the LAN coordinator being present. Environment writes must themselves
be atomic or redundant according to the bootloader's documented guarantees.

Partition-table format, partition sizes/alignment, filesystem type, read-only versus
writable mounts, redundancy for boot metadata, and placement of firmware/recovery
content are hardware-dependent choices. They must be selected with the target board,
bootloader, storage medium and endurance, secure-boot/signing requirements, and
firmware boot requirements in hand. Therefore this document recommends the A/B plus
persistent-data pattern but intentionally does **not** finalize an exact partition
table or filesystem layout until those target facts are known and validated.

### Apple TV 1st-generation (A1218 / AppleTV1,1) boot reality

The silver 1st-generation Apple TV is not a generic UEFI/Buildroot appliance. Public
boot documentation and community loader code describe an Apple EFI system that accepts
Apple's `boot.efi` path rather than an arbitrary EFI binary. The documented Linux path
is a compatibility chain: Apple `boot.efi` loads a file named `mach_kernel`; that file
is a wrapped 32-bit Linux kernel plus (typically) a gzip initramfs, with a dummy
extension used to satisfy the Apple loader. Community documentation also describes USB
boot-media discovery and a GPT disk with a FAT32 boot partition marked with the
Apple-TV recovery/boot partition type (`atvrecv`), with fallback to the internal drive
when no usable USB boot is found. These are evidence of an existing community path,
not a guarantee for every A1218 firmware revision or drive. See the [Apple TV Linux
bootloader](https://github.com/JaytheSpazz/appletv1-linux-bootloader), [modern Linux
loader notes](https://github.com/DistroHopper39B/atv-linux-loader), and [AppleTV1,1
boot-process documentation](https://github.com/macbian-admin/appletv1-boot-documentation).

**Practical feasibility:** a purpose-built 32-bit-compatible Linux/Buildroot system is
technically plausible only after validating this exact boot chain, kernel/device
support, graphics/display output, remote input, storage, and Kodi boundary on real
hardware. A normal Buildroot image cannot simply be flashed to an A1218 or assumed to
boot from its internal disk: Buildroot produces Linux artifacts, but does not supply
Apple's accepted first-stage loader, the `mach_kernel` compatibility wrapper, the
expected boot metadata, or a target-tested partition/firmware contract. Treat A1218
support as **experimental / target-specific**, not generic appliance support. No
device-modification procedure is prescribed here.

**Likely boot sequence (to be confirmed on target):** (1) power-on firmware/EFI
performs platform checks and looks for the Apple boot path on qualifying USB media; if
none is usable, it proceeds to the internal drive's Apple boot path; (2) Apple
`boot.efi` loads the Apple-style boot files and `mach_kernel`; (3) the wrapper hands
off to a 32-bit Linux kernel and initramfs; some existing Linux distributions then
use `kexec` to load a later kernel; (4) Linux mounts root and persistent data, starts
the client/Kodi boundary, and only then reports health. Exact selection rules,
internal-drive partition map, boot-file locations, `atvrecv` interpretation, kernel
arguments, and `kexec` use are **Unknown until observed on an A1218**.

**A/B qualification:** A/B root filesystems remain the safe generic design, but are
not a credible assumption for this target's stock boot chain. The Apple first stage has
no documented slot selector, boot-attempt counter, or health-confirmed rollback
contract in the cited path; internal-disk capacity alone does not create atomic boot
selection. Replacing Apple-visible boot files or changing the internal partition map
would be target-specific and failure-prone, not a routine Buildroot update. The A1218
baseline is therefore **no in-place OTA activation** until a tested loader contract
demonstrates inactive staging, atomic selection, power-loss behavior, and recovery
independent of the LAN coordinator. A/B is a hypothetical later engineering option,
not a promise in the client UI or release notes.

### Update screens and interaction

The update flow uses these explicit screens/states; each is a full-screen route with a
single focus scope and a clear primary action:

1. **Update available** — current version, candidate version, release size, source, and
   a plain-language statement that this is a whole Buildroot system update and will
   reboot the appliance. Focus starts on **View details** or **Update** when the
   candidate is already verified; **Later** is always reachable.
2. **Details and readiness** — release identity, compatibility, signature and integrity
   result, required free space, power/network requirements, and recovery behavior.
   **Update** is disabled until the image is complete, compatible, verified, and staged.
3. **Staging** — download and write progress for the inactive slot or equivalent
   staging area. Show determinate percentage/bytes when available; otherwise show an
   indeterminate activity state plus the current phase. The running system remains usable.
4. **Ready to restart** — verified candidate summary, expected downtime, and a two-step
   confirmation: **Restart and update** / **Not now**. The second confirmation requires
   Select/OK and names the reboot consequence.
5. **Preparing to restart** — the activation selection is committed atomically, then
   the screen shows a short countdown and “Do not remove power.” No ordinary remote action
   cancels or reverses this state once activation begins.
6. **Activating** — boot-time/full-screen progress with phase text such as “Checking
   image,” “Switching system,” and “Starting services.” It must never promise completion
   from download progress alone.
7. **First boot and health check** — the new system reports service, storage, display,
   remote-input, and Kodi-boundary checks. Show bounded progress and do not expose normal
   navigation until the minimum health gate passes.
8. **Complete** — identify the active version and offer **Continue**. Only after the
   health gate passes may the UI report success and return to Home/Kodi.
9. **Recovery / rolled back** — explain that the candidate did not pass first-boot
   health checks, identify the restored known-good version when available, and offer
   **Continue on previous version**, **Retry later**, and **Diagnostics**.
10. **Unrecoverable recovery** — show the documented recovery path if neither image is
    bootable; do not offer a normal retry that could loop.

### Remote navigation and focus during update

- Focus is trapped inside the current update screen; Kodi and ordinary playback controls
  cannot receive input.
- Up/Down moves between stacked actions or phases; Left/Right moves within a row.
  Navigation does not wrap. Disabled actions are skipped and their reason is text.
- Select/OK activates only the focused action. Update/reboot activation requires the
  explicit confirmation sequence; highlighting never starts work.
- Menu/Back closes details or returns to the previous update screen before staging.
  Once activation or reboot preparation starts, it is ignored or only shows the
  non-cancelable status; it must not interrupt activation.
- The focus ring, title, current phase, and primary action remain visible at 10-foot
  distance. Status is conveyed by text and shape/labels, not color alone.
- Remote loss, repeated Select, and power-button ambiguity must not create duplicate
  activation requests.

### Progress, reboot, and first-boot safety

Progress is phase-based and honest: **Checking → Downloading → Verifying → Staging →
Ready → Activating → Health check → Complete/Recovery**. Percentages are shown only
when backed by measured work; otherwise use phase text and an indeterminate indicator.
The coordinator persists an update journal with candidate identity, verification result,
staging location, activation intent, and boot attempt.

Before activation, cancellation must leave the current system bootable and usable. The
image must be fully received, signature/trust-chain verified, integrity checked,
compatible with the target, and staged in an inactive slot or equivalent recovery-safe
area. Require explicit confirmation and adequate power (or a platform-equivalent
safeguard); avoid rebooting when required network or storage conditions are uncertain.

After **Restart and update** is confirmed, activation is a no-interruption critical
section: do not accept remote commands that could cancel it, do not power off under
software control, and do not report success until the first-boot health gate passes.
Power loss during staging or activation must resolve to either the prior known-good
image or a complete candidate through documented boot recovery. A failed health check
rolls back or enters documented recovery; automatic retries are bounded.

### State machine

```text
Unknown / checking → Up to date
                   └→ Available
Available → Details → Downloading → Verifying → Ready to activate
                                  └───────────→ Failed (retry / details)
Ready to activate → Confirmed → Rebooting → Boot health check → Installed / Healthy
                                      └→ Failed health check → Rollback / recovery
```

The full-screen update flow contains: current system version, candidate system version,
artifact/source identity, verification state, progress when known, and the actions
**Update**, **Later**, and **Details**. **Update** is never automatic unless a
separately defined and disclosed appliance policy explicitly permits unattended
activation. A release check may be policy-controlled, but the project's `update.rs`
convention is authoritative for its current behavior: check state is `Unknown` until
a response, failed checks remain non-fatal/unknown, download begins only after user
action, and installation is only available for an AppImage run. The appliance
proposal must not copy that AppImage assumption to an unconfirmed OS.

The recommended activation design is an atomic system-image update: stage the signed
and trusted image into an inactive slot (or an equivalent platform-supported atomic
mechanism), verify its signature, trust chain, integrity, compatibility, and complete
payload before changing the boot/activation selection, then activate it on reboot.
A/B slots are recommended, not assumed; for A1218 the Apple EFI/`mach_kernel` chain
has not been shown to provide slot selection or rollback semantics. Until that evidence
exists, A1218 has no supported internal-storage OTA activation; any alternative must
provide an equally explicit inactive/staged area, atomic switch, interruption-safe
recovery, and a known-good previous system. Never overwrite
the currently running system in place unless the target platform's recovery guarantees
are documented and accepted.

Activation requires an explicit user confirmation after the candidate is verified,
except for the separately defined policy above. Before confirmation, the UI must show
that this is a whole-system update, the expected reboot, the power/network
requirements, and the recovery behavior. The coordinator must defer activation when
power is not safely available or the network is needed but unreliable; it must not
begin a reboot with an incomplete image, missing verification, or an uncertain write.
The reboot UX must provide a clear countdown/confirmation, prevent accidental
interruption where possible, explain that Kodi and the client will stop, and show a
pending/activating state on the next boot rather than implying completion before
health is known.

While the full-screen update flow is visible:

- focus is trapped inside the full-screen flow;
- Menu/Back chooses **Later** or closes the details layer, not a partial install;
- media files, the sync manifest, and Kodi's database are not mutated by update work;
- the whole-system image is staged and verified before activation;
- cancellation leaves the running version usable;
- power-loss/network-loss handling leaves either the running system or the complete
  staged candidate recoverable;
- activation/reboot wording matches the platform's verified behavior; and
- after reboot, a boot health check confirms the system, client, storage, remote
  control, and required Kodi handoff path. If health is not confirmed within the
  defined window, the boot system must roll back to the prior known-good system (or
  enter the documented recovery path); the UI must not report success before that
  check passes.

If the client cannot safely appear above Kodi, it must not pretend that it did: show
an actionable client notification at the next available boundary and record the
update as pending. Do not infer successful activation from a downloaded file alone.

## 5. Release lifecycle and stable-channel policy

The appliance follows a release lifecycle that separates the weekly engineering
cadence from what devices are allowed to install. **v1 is the first stable build**:
it is the first build that has passed the target release gate, has a signed immutable
artifact and manifest, and is intentionally published to the stable channel. A
weekly build is not automatically a release and must never become installable merely
because it was produced.

### Versioning and release records

- Use a user-facing semantic release version for stable artifacts: `MAJOR.MINOR.PATCH`
  (for example, `1.0.0` for the first stable build). Increment `MAJOR` for an
  incompatible appliance contract, `MINOR` for compatible capabilities, and `PATCH`
  for compatible fixes. The exact compatibility policy remains subject to the target
  contract, but version comparison must be deterministic and must not rely on display
  strings.
- Give every produced artifact a distinct immutable build identity, including the
  source revision, build timestamp or monotonic build number, target compatibility
  identity, and artifact digest. Weekly candidates may carry prerelease/build metadata
  such as `1.1.0-rc.1+build.20261021`; build metadata is diagnostic and must not be
  treated as a stable release.
- A stable release record binds the release version to exactly one signed manifest and
  artifact digest, compatibility constraints, release notes, minimum recovery
  requirements, and the validation evidence that approved it. Never replace the
  bytes behind a published version; supersede it with a new version and retain the
  old manifest/digest for rollback and audit.
- The first stable record is `v1.0.0` unless the approved target versioning contract
  explicitly requires another valid `1.0.0`-equivalent representation. The UI and
  diagnostics show both the stable release version and the immutable build identity.

### Weekly build, staged release, and promotion

After v1, produce one new build each week (or record why a week was skipped). The
weekly build enters a non-stable validation channel and follows this promotion path:

1. **Build:** create a reproducible, signed candidate with manifest, digest,
   compatibility metadata, release notes, and a unique build identity. Signing a
   candidate proves provenance and integrity; it does not make it stable.
2. **Internal validation:** run automated checks and the relevant target/fixture
   matrix, including staging, power/network interruption, boot health, rollback,
   coordinator loss, LAN loss, and UI/Kodi boundary checks. Reject the candidate on
   any release-gate failure; do not expose a rejected candidate to devices.
3. **Staged test release:** publish the candidate to a test/preview channel with an
   explicit channel label and bounded test cohort (for example, lab devices or
   opted-in paired devices). Test devices may retrieve it only after an explicit
   test-channel policy and must display that it is not stable. Collect health,
   installation, rollback, and user-visible failure evidence for a defined soak
   period.
4. **Promotion decision:** an owner reviews the recorded evidence, open blockers,
   compatibility, and rollback readiness. Only an approved candidate is promoted by
   publishing a new signed stable-channel index/manifest. Promotion is an explicit
   action, not a timer or the existence of a weekly build.
5. **Stable rollout:** stable devices become eligible for the promoted release under
   the rollout policy. The stable index may use a bounded percentage or cohort rollout,
   but every eligible device resolves to a version explicitly marked **Stable**. The
   latest weekly candidate remains invisible to stable-channel clients until promoted.

A failed or inconclusive validation keeps the previous stable release selected; the
weekly build may be fixed or superseded the following week. There is no requirement
to publish a broken weekly candidate, and a skipped weekly build does not advance the
stable channel.

### Stable selection and distribution

The coordinator and device use a channel-aware release index, not filename sorting or
build date alone. Normal devices are pinned to **Stable** and select the highest
compatible, non-revoked release explicitly marked stable by the trusted publisher.
They must not retrieve, stage, or install a preview, test, release-candidate, or merely
newer weekly build while on the stable channel. A user-visible channel change to
preview/test is an explicit, separately authorized diagnostic/developer policy and is
out of the normal stable path; changing it does not silently change the device's
trust, compatibility, or rollback rules.

The coordinator may cache or obtain the stable artifact and distribute it to paired
devices over the local LAN where applicable. LAN distribution is transport only: the
device independently authenticates the coordinator session and verifies the signed
manifest, artifact digest, target compatibility, release channel, complete payload,
and revocation/expiry policy before staging. A coordinator must not promote a weekly
build by serving it locally, and a device must not trust a manifest merely because it
came from a paired coordinator. A device that cannot reach the release source may use
an already cached stable artifact or an approved LAN copy, but reports **Pending** or
**Unavailable** when it cannot establish that a newer stable release is available; it
must never report **Up to date** based on a failed check.

For multiple devices, the coordinator resolves the stable selection separately for
each device's compatibility identity, records the selected stable version/digest per
device, and does not claim installation until the device's durable health result is
acknowledged. Transfer, staging, and activation retain the existing resumable,
idempotent, device-owned flow in Section 1 and the update state machine above.

### Rollout, rollback, and channel safeguards

- Roll out in bounded cohorts where the target supports it, with a pause/hold switch
  that prevents new activations without invalidating the currently running system.
  Monitor install success, first-boot health, rollback rate, coordinator/LAN errors,
  and critical user-visible regressions before widening the cohort.
- Automatically halt promotion/rollout when predefined safety thresholds are exceeded
  or when signing, manifest, compatibility, or health evidence is missing. A release
  can be revoked in the trusted index; revocation blocks new staging while the device
  retains its known-good recovery path.
- The device uses the existing atomic inactive-slot/equivalent activation and bounded
  boot-health gate. A failed candidate returns to the previous known-good stable
  system (or documented recovery), records the candidate version/build and reason,
  and reports **Rolled back**. Rollback is a device safety outcome, not proof that the
  coordinator completed or that the candidate is stable.
- On rollback or a rollout pause, stable selection falls back to the last approved
  stable release for that device. Do not automatically retry a failed candidate in a
  loop; require a new promotion decision or explicit, safe retry policy. Preserve
  manifests, digests, journals, and recovery evidence for diagnosis.
- The coordinator may offer **Update available** only for a verified stable candidate
  selected for that device. Downloading, staging, and activation remain separate
  states; normal stable updates still require the existing explicit confirmation and
  first-boot health gate unless a separately approved appliance policy says otherwise.

### Roadmap fit

Stage 0 defines the semantic version/build identity, signing and immutable manifest
contract, channel trust/promotion rules, stable-index format, cohort/hold thresholds,
and retention/revocation policy. Stage 2 produces the reproducible weekly artifact,
manifest, and test-channel publication inputs while preserving the current system.
Stage 3 and Stage 4 enforce device-owned activation, health confirmation, and rollback
for a promoted stable release. Stage 5 adds the weekly cadence, staged test cohort,
stable promotion approval, LAN distribution, rollout monitoring, and a release record
for `v1.0.0` and each subsequent stable version. No stage may claim release support
until M7 covers the channel-selection, promotion, rollback, and multi-device LAN
acceptance paths.

## 6. Staged implementation roadmap

This roadmap turns the update design into verifiable increments without making the
unconfirmed target choices look settled. Each stage has a demonstrable outcome and an
exit gate. Work may be developed with host-side fixtures first, but a stage is not
accepted as appliance support until its target-specific gate passes on the running
Buildroot image and remote/display hardware.

### Stage 0 — Target and recovery contract

**Goal:** close the minimum integration boundary before writing activation code.

- Define the local-LAN topology and trust boundary: coordinator placement, eligible
  device networks, discovery behavior across sleep/address changes, pairing ceremony,
  persistent identities, unpair/revocation behavior, offline expectations, and the
  failure behavior when the LAN disappears. Leave ports and wire protocols **Unknown**
  until the selected implementation is tested.
- Define the coordinator/device operation model: durable operation identifiers,
  idempotent retries, device-owned update verification and boot/recovery authority,
  transfer resume semantics, coordinator loss during staging, LAN loss during reboot,
  and journal reconciliation after reconnection.
- Record the target board/CPU (including A1218 / AppleTV1,1 identity), display stack,
  remote input path, power signal, Apple EFI/`boot.efi` loader chain, storage layout,
  Buildroot release/configuration, and Kodi launch/foreground behavior. For A1218,
  explicitly document USB/internal-drive selection, `mach_kernel` wrapping, and any
  `kexec` use; do not infer these from a generic Buildroot build.
- Select the signed Buildroot artifact format and update source. Define the trust
  anchor, key rotation/revocation policy, compatibility identity, version scheme, and
  minimum free-space/power requirements. Define the semantic release version,
  immutable build identity, stable/test channel metadata, promotion authority, rollout
  hold thresholds, revocation, and artifact retention policy.
- Choose A/B slots or document the equivalent inactive staging area, atomic boot
  selection, boot-attempt counter, known-good marker, rollback trigger, and manual
  recovery path. For A1218, this is a feasibility gate: absent a tested Apple-boot-chain
  solution, document **no OTA activation / external recovery only** rather than inventing
  A/B semantics. Define the maximum health-check window and bounded retry count.
- Define the update journal schema and persistence location. A power loss must leave
  enough evidence to distinguish **not staged**, **staged**, **activation pending**,
  **booting candidate**, **healthy**, and **rolled back**.

**Exit gate:** a target-specific recovery runbook exists; an unsigned, corrupt,
wrong-target, and interrupted image have defined outcomes; and the boot/recovery owner
has approved the rollback contract. Until this gate passes, the client may show mock
states only and must not activate a real image.

### Stage 1 — Full-screen update UI shell and remote contract

**Goal:** build the Apple TV-style living-room interaction without coupling it to
unsafe update operations. “Apple TV-style” here means full-screen, sparse, calm,
10-foot-readable presentation and remote-first focus behavior—not copied branding or
an exact product replica.

- Implement the full-screen route and state model: **Update available**, **Details and
  readiness**, **Staging**, **Ready to restart**, **Preparing to restart**, **Activating**,
  **First boot and health check**, **Complete**, **Recovery / rolled back**, and
  **Unrecoverable recovery**.
- Implement focus trapping, visible focus ring, deterministic initial focus, disabled
  actions with reasons, explicit **Update** and **Restart and update** confirmations,
  Menu/Back layering, repeated-Select protection, and non-cancelable activation wording.
- Use a fake update provider and deterministic fixtures for progress, stalled work,
  verification failure, reboot-pending, health failure, and rollback. Keep Kodi and
  normal playback input out of the full-screen route.
- Persist and restore the user-visible update state so a restart never presents a
  stale “complete” message. Show whole-system/Buildroot language, expected downtime,
  power requirements, and recovery behavior.

**Exit gate:** remote navigation and accessibility checks pass against the UI fixture;
all terminal states are reachable without hidden gestures; no fixture state claims
activation or success before a health result; and the screen remains legible at the
agreed 10-foot distance.

### Stage 1A — Local-LAN control plane and pairing

**Goal:** establish a safe coordinator/device control plane before transferring or
activating real images.

- Implement fixture-driven discovery, stale/duplicate handling, address changes,
  device identity, explicit pairing confirmation, unpair/revocation, and the visible
  states **Discovered**, **Pairing required**, **Paired but unreachable**,
  **Authenticated**, and **Failed**. Do not treat discovery as authorization.
- Implement mutually authenticated, encrypted sessions using the target design
  selected after review, with replay protection, capability negotiation, bounded
  timeouts, and credentials excluded from logs. Until that review and target test are
  complete, keep the LAN port assignment, transport, and wire protocol undecided; do
  not bake any fixed value into the UI or contract.
- Implement durable, idempotent operation IDs and reconciliation for accepted,
  in-progress, completed, rejected, expired, and unknown outcomes. Exercise coordinator
  loss, device sleep, LAN loss, reconnect, repeated commands, and stale responses.
- Prove offline/no-cloud behavior with a local paired device and an already available
  artifact. Internet absence must not block local control or cause an **Up to date**
  claim; unreachable devices must not be reported as staged, rebooted, or healthy.

**Exit gate:** pairing requires deliberate user confirmation; authenticated traffic is
confidential and integrity-protected; retries do not duplicate operations; and target
tests show safe behavior across discovery, reconnect, coordinator loss, and LAN loss.
No update activation is enabled by this stage.

### Stage 2 — Buildroot image pipeline and safe staging

**Goal:** produce and stage a real, trusted appliance image while the current system
remains usable.

- Make the Buildroot build reproducible enough to emit a versioned image, manifest,
  compatibility metadata, signature, and published checksums. Include the client,
  runtime dependencies, configuration defaults, and recovery tooling in the image.
- Add the client-side update coordinator for check, download, resume/retry, free-space
  check, signature/trust-chain verification, integrity verification, compatibility
  verification, and staging to the inactive slot or approved equivalent.
- Write journal transitions atomically and make download/staging idempotent. Never
  mutate media files, the sync manifest, or Kodi's database. Do not overwrite the
  running system in place.
- Exercise network loss, storage removal/read-only storage, insufficient space,
  corrupt/truncated payloads, wrong hardware identity, expired/revoked trust, and
  power loss during staging. Keep the existing system bootable in every pre-activation
  failure.

**Exit gate:** a real signed image can be downloaded and staged on target; every
negative case leaves the current version usable; the UI exposes accurate measured
progress or an indeterminate phase; and the journal can reconstruct the operation.

### Stage 3 — Explicit reboot and atomic activation

**Goal:** activate only a fully verified image after deliberate user confirmation.

- Enable **Restart and update** only when staging and all required checks are complete.
  Present the candidate version, whole-system scope, expected downtime, power/network
  requirements, and rollback behavior in the confirmation step.
- On confirmation, atomically record activation intent and select the candidate slot
  (or equivalent), show the countdown and “Do not remove power,” stop the client/Kodi
  cleanly according to the target contract, and reboot. Ignore ordinary remote input
  once the non-cancelable critical section starts.
- On the next boot, identify the pending candidate and show **Activating** rather than
  claiming completion. The boot system must count attempts and automatically choose the
  known-good image when the candidate cannot reach the health gate.
- Validate interrupted activation and power loss at each available boundary. A reboot
  request must be idempotent: repeated remote events cannot create a second activation
  or corrupt the boot selection.

**Exit gate:** a signed candidate activates only after explicit confirmation; the
system returns to either a complete candidate or the prior known-good image after
controlled interruption and power-loss tests; and the observed reboot wording matches
the target behavior.

### Stage 4 — First-boot verification and health gate

**Goal:** report success only when the new appliance is usable, not merely bootable.

- Run bounded checks for image/version identity, required services, client startup,
  display output, remote input, writable persistent storage, managed media root, and the
  selected Kodi launch/foreground/handoff path. Record each check as pass, fail, or
  not-applicable with an actionable reason.
- Keep the UI in **First boot and health check** until the minimum gate passes. Do not
  expose normal Home/Kodi navigation, mark the image known-good, or delete the prior
  recovery path before that point.
- On a failed or timed-out gate, stop retrying after the defined bound, preserve
  diagnostics, mark the candidate unhealthy, and request rollback. The recovery screen
  must name the restored version when known and offer **Continue on previous version**,
  **Retry later**, and **Diagnostics**.
- After a successful gate, atomically mark the candidate known-good, retain the prior
  image for the defined rollback window, and transition to **Complete**. Reconcile the
  journal so a later restart cannot replay activation.

**Exit gate:** injected service, display, remote, storage, and Kodi-boundary failures
produce the documented recovery path; a healthy boot reaches Home only after all
minimum checks pass; and the active version shown by the UI matches the verified boot
slot.

### Stage 5 — Real-target validation milestones and release readiness

**Goal:** prove the complete flow under realistic living-room and failure conditions.

1. **M1 — UI contract:** fixture-driven tests cover focus, Menu/Back, confirmations,
   progress, accessibility names/states, repeated input, and every terminal state.
2. **M2 — Buildroot artifact:** a clean build produces a signed image and manifest;
   signature, compatibility, integrity, and version checks reject negative fixtures.
3. **M3 — Staging soak:** target tests cover resume, network loss, storage loss,
   read-only storage, insufficient space, and power loss before activation; the running
   version remains usable.
4. **M4 — Reboot/rollback:** target tests cover explicit confirmation, countdown,
   interrupted activation, candidate boot failure, health timeout, automatic rollback,
   recovery diagnostics, and bounded retries.
5. **M5 — Living-room acceptance:** remote navigation, 10-foot readability, display
   foregrounding above Kodi or the documented fallback, Kodi ownership boundaries,
   platform Home/TV pass-through, and return to the client are observed on hardware.
6. **M6 — Release gate:** repeat the matrix on the release candidate, review logs for
   secrets, verify the recovery runbook, record known limitations, and obtain sign-off
   from the appliance/boot, UI, and Kodi-boundary owners. Confirm that the candidate
   is either promoted as stable or remains test-only; exercise stable-channel selection,
   bounded rollout/hold, rollback, and coordinator LAN distribution.
7. **M7 — Release operations:** publish v1 as the first stable build, then exercise the
   weekly build → staged test → explicit stable promotion workflow with a non-stable
   candidate. Confirm that stable devices never select the unvalidated weekly build.
   Record release evidence, rollout metrics, rollback/revocation actions, and the
   stable manifest for each promoted version.

A milestone may be marked **Passed** only with recorded target, image/version, test
fixture or failure injection, journal evidence, observed result, and recovery result.
A host fixture can validate coordinator logic but cannot satisfy M4 or M5. Until M6
passes, release notes and the UI must describe the feature as **Planned**, **Testing**,
or **Recovery required**, rather than supported.

## 7. Accessibility and error states

Accessibility is a design requirement, not a later visual pass. Every focusable item
has a stable accessible name, role, current value/state, and enabled/disabled state.
The focus ring has sufficient contrast against every supported surface and is visible
without relying on hover. Menus and dialogs announce their title and the focused item;
progress exposes determinate/indeterminate state and an approximate description when
available. Text alternatives accompany icons and remote-only affordances.

Errors use the same explicit-state discipline as the existing project:

| Condition | User-visible state | Safe action |
| --- | --- | --- |
| No drive | Not detected; say what to connect/check | Retry detection, Settings, Diagnostics |
| Read-only/vanished/unknown drive | Degraded; no write is attempted | Reconnect/eject or inspect diagnostics |
| Insufficient space | Plan blocked with required/available space | Cancel, change target, retry |
| Sync interruption | Paused/failed with journal status | Resume or abandon; never expose a partial file as complete |
| Kodi boundary unavailable | Kodi unavailable / refresh unconfirmed | Retry, open diagnostics, continue local management |
| Update check unavailable | Unknown, not “up to date” | Retry or leave the appliance usable |
| Verification/activation failure | Failed; running version unchanged | Retry, details, recovery path |
| Unsupported remote action | No-op with discoverable explanation where useful | Use supported controls; do not map silently to another action |

Error text names the affected operation and the next safe action. Logs and diagnostics
must not expose credentials, and must not turn a network or Kodi failure into a claim
that media or updates are current.

## 8. Unconfirmed decisions / acceptance gate

The following remain **Unknown** and are intentionally not designed as facts:

- coordinator/device LAN topology, discovery mechanism, pairing ceremony, identity
  and revocation model, authenticated encrypted transport, port assignment, wire
  protocol, and offline behavior;
- appliance OS, CPU, display stack, boot/activation model, and remote hardware; for
  A1218 specifically, Apple EFI/`boot.efi` acceptance, `mach_kernel` wrapping,
  USB/internal-drive selection, partition map, and `kexec` use;
- whether the client is headless, a shell, or a foreground UI alongside Kodi;
- Kodi version and supported add-on/API/IPC mechanism;
- how Kodi is launched, focused, paused, refreshed, or returned from;
- foreground/full-screen behavior above Kodi;
- sync direction, managed layout, filesystem, and conflict policy;
- update source, trusted signing/key policy, Buildroot image/artifact format,
  inactive-slot or equivalent atomic activation mechanism, power/network safeguards,
  reboot UX, boot health-check window, and rollback/recovery behavior;
- exact accessibility bridge available on the target.

Before implementation is marked planned, acceptance must include a real-target test
matrix: remote focus movement and Menu/Back layering; handoff acknowledgement and
failure; Kodi playback ownership; platform Home/TV pass-through; full-screen update flow and documented foreground fallback;
interrupted sync; read-only/vanished storage; unsigned/untrusted/corrupt or incompatible
whole-system image; interrupted staging and power/network loss; explicit confirmation
and reboot UX; activation failure; failed boot health check with rollback; and
screen-reader/contrast checks. A directory fixture may validate planner
and recovery logic, but cannot claim hardware, Kodi, remote, or full-screen foreground support.

## Consistency check against the project files

- Matches `README.md` and `ROADMAP.md`: Rust/native/local-first conventions, explicit
  playback and failure states, diagnostics, persisted settings, and no unconfirmed
  feature presented as working.
- Preserves the roadmap's distinction between **Planned** work and **Unknown**
  device/integration decisions.
- Retains the appliance conventions already established by the project: drive
  lifecycle, dry-run sync, managed roots, Kodi ownership, staged/verified whole-system
  updates, reboot health checking/rollback, and open decisions.
- Defines the coordinator as a local-LAN control-plane role while keeping discovery,
  pairing, transport, ports, and wire protocol unselected until target validation; LAN
  distribution remains transport-only and cannot promote an unvalidated weekly build.
- Adds a versioned release lifecycle: v1 as the first stable build, weekly candidates
  through staged validation, explicit stable promotion, stable-only device selection,
  bounded rollout, and device-owned rollback.
- Treats `update.rs` as an implementation reference only where it is explicit: async
  check, non-fatal `Unknown`, user-requested download, staged install, and AppImage-
  specific activation. It does not generalize AppImage behavior to the appliance.
