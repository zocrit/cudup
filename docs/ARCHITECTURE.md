## 1. High-Level Design Philosophy

`cudup` differs from standard NVIDIA installers in three key ways:
1.  **User-Space First:** No `sudo` required. Everything lives in `~/.cudup` (or `$XDG_DATA_HOME`).
2.  **Manifest Driven:** We do not hardcode versions. We scrape NVIDIA's "Redist" (redistributable) servers to discover versions dynamically.
3.  **Component-Based:** We do not download the monolithic `.run` installers. We download granular components (nvcc, cudart, cuDNN) and assemble them locally.

### System Diagram

```mermaid
graph TD
    %% --- Styles ---
    classDef rust fill:#dea584,stroke:#333,stroke-width:2px,color:black;
    classDef fs fill:#f9f9f9,stroke:#666,stroke-width:2px,stroke-dasharray: 5 5,color:black;
    classDef remote fill:#90CDF4,stroke:#333,stroke-width:2px,color:black;

    %% --- Actors & Entry ---
    User((User))
    CLI[CLI Entrypoint<br/>clap]:::rust

    %% --- The Core Application ---
    subgraph Core [Cudup Rust Binary]
        direction TB
        Orchestrator{Command<br/>Handler}:::rust
        
        subgraph Network [Network Layer]
            ManifestFetcher[<b>Manifest Fetcher</b><br/>Scrapes redist directory]:::rust
            ArtifactDownloader[<b>Parallel Downloader</b><br/>reqwest + tokio]:::rust
        end

        subgraph LocalOps [Local Operations]
            Installer[<b>Installer</b><br/>Unpacks tarballs]:::rust
            LinkManager[<b>Symlink Manager</b><br/>Updates active version]:::rust
        end
    end

    %% --- Remote Resources ---
    subgraph Nvidia [NVIDIA Developer Zone]
        RedistIndex[Redist Index HTML]:::remote
        JSONManifests[JSON Manifests]:::remote
        Tarballs[Component Tarballs]:::remote
    end

    %% --- Local File System ---
    subgraph FileSystem [~/.cudup]
        CacheDir[<b>/cache</b><br/>Raw .tar.xz files]:::fs
        VersionsDir[<b>/versions</b><br/>/11.8<br/>/12.0]:::fs
        CurrentLink[<b>/current</b><br/>Symlink to active]:::fs
        BinShim[<b>/bin</b><br/>shim executables]:::fs
    end

    %% --- Data Flow ---
    User --> CLI
    CLI --> Orchestrator
    Orchestrator --> ManifestFetcher
    ManifestFetcher --> RedistIndex
    ManifestFetcher --> JSONManifests
    JSONManifests --> ArtifactDownloader
    ArtifactDownloader --> Tarballs
    Tarballs --> CacheDir
    CacheDir --> Installer
    Installer --> VersionsDir
    Orchestrator --> LinkManager
    LinkManager --> CurrentLink
    CurrentLink -.-> VersionsDir