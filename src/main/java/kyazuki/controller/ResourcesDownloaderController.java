package kyazuki.controller;

import java.io.IOException;
import java.util.logging.Level;
import java.util.logging.Logger;

import javafx.concurrent.Service;
import javafx.concurrent.Task;
import javafx.concurrent.WorkerStateEvent;
import javafx.event.EventHandler;
import javafx.fxml.FXML;
import javafx.scene.control.Label;
import javafx.scene.control.ProgressBar;
import kyazuki.App;
import kyazuki.dataclass.Config;
import kyazuki.dataclass.Config.Resource;

public class ResourcesDownloaderController {
    @FXML
    private Label resourceNameLabel;
    @FXML
    private ProgressBar progressBar;

    @FXML
    public void initialize() {
        Service<Void> service = new Service<>() {
            @Override
            protected Task<Void> createTask() {
                return new Task<>() {
                    @Override
                    protected Void call() throws Exception {
                        Logger logger = App.getLogger();
                        Config config = App.getConfig();
                        if (config.resources != null) {
                            // リソース群をダウンロードする
                            logger.info("Start downloading resource...");
                            for (int i = 0; i < config.resources.length; i++) {
                                Resource resource = config.resources[i];
                                updateMessage(resource.name);
                                try {
                                    if (resource.download()) {
                                        logger.info("\tDownloaded: " + resource.name);
                                    } else {
                                        logger.info("\tSkipping download: " + resource.name);
                                    }
                                    updateProgress(i + 1, config.resources.length);
                                } catch (IOException e) {
                                    logger.log(Level.SEVERE, "Failed to download resource: " + resource.name, e);
                                    throw e;
                                }
                            }
                        }
                        return null;
                    }
                };
            }
        };
        resourceNameLabel.textProperty().bind(service.messageProperty());
        progressBar.progressProperty().bind(service.progressProperty());
        service.setOnSucceeded(new EventHandler<WorkerStateEvent>() {
            @Override
            public void handle(WorkerStateEvent event) {
                try {
                    App.setRoot("profileAppender");
                } catch (IOException e) {
                    App.getLogger().log(Level.SEVERE, "Failed to transit profileAppender view.",
                            e);
                    App.showAlertAndExit("ビュー遷移に失敗しました。");
                }
            }
        });
        service.setOnFailed(new EventHandler<WorkerStateEvent>() {
            @Override
            public void handle(WorkerStateEvent event) {
                App.showAlertAndExit("リソースのダウンロードに失敗しました。");
            }
        });
        service.start();
    }
}
