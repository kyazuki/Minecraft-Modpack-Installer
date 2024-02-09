package kyazuki;

import java.io.File;
import java.io.FileInputStream;
import java.io.FileWriter;
import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.time.Instant;
import java.util.Locale;
import java.util.UUID;
import java.util.logging.FileHandler;
import java.util.logging.Handler;
import java.util.logging.Level;
import java.util.logging.LogManager;
import java.util.logging.Logger;
import java.util.logging.SimpleFormatter;

import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.databind.SerializationFeature;
import com.fasterxml.jackson.dataformat.yaml.YAMLFactory;
import com.fasterxml.jackson.datatype.jsr310.JavaTimeModule;

import kyazuki.Profiles.Profile;

public class Main {
    public static void main(String[] args) {
        // ロガーを準備する
        LogManager.getLogManager().reset();
        Logger logger = Logger.getLogger("minecraft-modpack-installer");
        Handler handler = null;
        try {
            handler = new FileHandler("installer.log");
            handler.setEncoding("UTF-8");
        } catch (SecurityException | IOException e) {
            e.printStackTrace();
            System.exit(1);
        }
        logger.addHandler(handler);
        Locale.setDefault(new Locale("en", "EN"));
        System.setProperty("java.util.logging.SimpleFormatter.format",
                "%1$tY-%1$tm-%1$td %1$tH:%1$tM:%1$tS.%1$tL %4$s [%2$s] %5$s%6$s%n");
        handler.setFormatter(new SimpleFormatter());
        // 設定ファイルを読み込む
        File f = new File("config.yaml");
        if (!f.exists()) { // 設定ファイルが見つからないとき
            logger.severe("No config file found: " + f.getPath());
            System.exit(1);
        }
        Config config = null;
        try (FileInputStream fis = new FileInputStream(f)) {
            ObjectMapper yaml = new ObjectMapper(new YAMLFactory());
            config = yaml.readValue(fis, Config.class);
        } catch (IOException e) {
            logger.log(Level.SEVERE, "Failed to load config file.", e);
            System.exit(1);
        }
        // ModLoaderをダウンロードする
        try {
            config.modLoader.download();
        } catch (IOException e) {
            logger.log(Level.SEVERE, "Failed to download Mod Loader.", e);
            System.exit(1);
        }
        // Mod群をダウンロードする
        for (Config.CurseForgeMod mod : config.curseForgeMods) {
            try {
                mod.download();
            } catch (IOException e) {
                logger.log(Level.SEVERE, "Failed to download mod: " + mod.name, e);
                System.exit(1);
            }
        }
        // プロファイルを追加する
        PROFILE: {
            String os = System.getProperty("os.name");
            if (!os.startsWith("Windows")) { // Windows以外のOSのとき
                // プロファイルファイルのパスが不明なためスキップ
                logger.warning("'" + os + "' is not supported. Skip adding profile.");
                break PROFILE;
            }
            Path profilesPath = Path.of(System.getenv("APPDATA"), ".minecraft", "launcher_profiles.json");
            if (!Files.exists(profilesPath)) { // プロファイルファイルが見つからないとき
                logger.warning("No profile file found. Skip adding profile.");
                break PROFILE;
            }
            Profiles profiles = null;
            try (FileInputStream fis = new FileInputStream(profilesPath.toFile())) {
                ObjectMapper json = new ObjectMapper();
                json.registerModule(new JavaTimeModule());
                profiles = json.readValue(fis, Profiles.class);
            } catch (IOException e) {
                logger.log(Level.WARNING, "Failed to load profile file. Skip adding profile.", e);
                break PROFILE;
            }
            final String profileName = config.profile.name;
            if (profiles.profiles.entrySet().stream().anyMatch(e -> e.getValue().name.equals(profileName))) {
                // 既に同じ名前のプロファイルが存在するとき
                logger.warning("Profile already exists. Skip adding profile.");
                break PROFILE;
            }
            String uuid = null;
            while (true) {
                uuid = UUID.randomUUID().toString().replace("-", "");
                if (profiles.profiles.get(uuid) == null) {
                    break;
                }
            }
            Instant now = Instant.now();
            Profile newProfile = new Profile();
            newProfile.created = now;
            newProfile.gameDir = Path.of(".").toAbsolutePath().getParent().toString();
            newProfile.icon = config.profile.icon;
            newProfile.javaArgs = config.profile.javaArgs;
            newProfile.lastUsed = now;
            newProfile.lastVersionId = config.profile.versionId;
            newProfile.name = config.profile.name;
            newProfile.resolution = null;
            newProfile.type = "custom";
            profiles.profiles.put(uuid, newProfile);
            Path backupProfilesPath = profilesPath
                    .resolveSibling(profilesPath.getFileName().toString() + ".bak");
            for (int i = 1; Files.exists(backupProfilesPath); i++) {
                backupProfilesPath = profilesPath
                        .resolveSibling(profilesPath.getFileName().toString() + ".bak" + i);
            }
            if (!profilesPath.toFile().renameTo(backupProfilesPath.toFile())) {
                // プロファイルファイルのバックアップ(リネーム)に失敗したとき
                logger.warning("Failed to backup profile file. Skip adding profile.");
                break PROFILE;
            }
            try (FileWriter fw = new FileWriter(profilesPath.toFile())) {
                ObjectMapper json = new ObjectMapper();
                JavaTimeModule module = new JavaTimeModule();
                module.addSerializer(Instant.class, new Profiles.CustomInstantSerializer());
                json.registerModule(module);
                json.enable(SerializationFeature.INDENT_OUTPUT);
                json.disable(SerializationFeature.WRITE_DATES_AS_TIMESTAMPS);
                fw.write(json.writeValueAsString(profiles));
            } catch (IOException e) {
                logger.log(Level.WARNING, "Failed to save profile file. Skip adding profile.", e);
                break PROFILE;
            }
        }
    }
}